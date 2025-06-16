import { Alpine,  ElementWithXAttributes } from "alpinejs";
import { fetchEventSource, FetchEventSourceInit, EventSourceMessage } from "./fetch-event-source";
import { isBoolString } from "./utils";
import { docWithViewTransitionAPI, supportsViewTransitions } from "./view-transition";

export const STARTED = 'started'
export const FINISHED = 'finished'
export const ERROR = 'error'
export const RETRYING = 'retrying'
export const RETRIES_FAILED = 'retries-failed'

const isWrongContent = (err: any) => `${err}`.includes('text/event-stream')

export type SSEArgs = {
  payload?: Object,
  headers?: Record<string, string>
  openWhenHidden?: boolean
  retryInterval?: number
  retryScaler?: number
  retryMaxWaitMs?: number
  retryMaxCount?: number
  abort?: AbortSignal
} & (
  | {
      contentType: 'json'
    }
  | {
      contentType: 'form'
      selector?: string
    }
)

export enum FragmentMergeModes {
    // Morphs the fragment into the existing element using idiomorph.
    Morph = "morph",
    // Replaces the inner HTML of the existing element.
    Inner = "inner",
    // Replaces the outer HTML of the existing element.
    Outer = "outer",
    // Prepends the fragment to the existing element.
    Prepend = "prepend",
    // Appends the fragment to the existing element.
    Append = "append",
    // Inserts the fragment before the existing element.
    Before = "before",
    // Inserts the fragment after the existing element.
    After = "after",
    // Upserts the attributes of the existing element.
    UpsertAttributes = "upsertAttributes",
};

export function sse(alpine: Alpine & {morph?: (target: Element, fragment: Element) => void}) {
  function applyToTargets(
    mergeMode: string,
    fragment: Element,
    capturedTargets: Element[],
  ) {
    for (const target of capturedTargets) {
      // Mark the target as a fragment merge target to force plugins to clean up and reapply
      (target as HTMLElement).dataset.fragmentMergeTarget = 'true'

      // Clone the fragment to merge to avoid modifying the original and force browsers to merge the fragment into the DOM
      const fragmentToMerge = fragment.cloneNode(true) as  Element & (HTMLElement | SVGElement) 
      switch (mergeMode) {
        case FragmentMergeModes.Morph: {
          if(alpine.morph == undefined) {
            throw new Error('Alpine morph plugin is not loaded.') 
          }
          alpine.morph(target, fragmentToMerge);
          break
        }
        case FragmentMergeModes.Inner:
          // Replace the contents of the target element with the outer HTML of the response
          target.innerHTML = fragmentToMerge.outerHTML
          break
        case FragmentMergeModes.Outer:
          // Replace the entire target element with the response
          target.replaceWith(fragmentToMerge)
          break
        case FragmentMergeModes.Prepend:
          // Insert the response before the first child of the target element
          target.prepend(fragmentToMerge)
          break
        case FragmentMergeModes.Append:
          // Insert the response after the last child of the target element
          target.append(fragmentToMerge)
          break
        case FragmentMergeModes.Before:
          // Insert the response before the target element
          target.before(fragmentToMerge)
          break
        case FragmentMergeModes.After:
          // Insert the response after the target element
          target.after(fragmentToMerge)
          break
        case FragmentMergeModes.UpsertAttributes:
          // Upsert the attributes of the target element
          for (const attrName of fragmentToMerge.getAttributeNames()) {
            const value = fragmentToMerge.getAttribute(attrName)!
            target.setAttribute(attrName, value)
          }
          break
        default:
          throw 'InvalidMergeMode' 
      }
    }
  }

  function sse_dispatch_consumer(el: ElementWithXAttributes, evt: EventSourceMessage) {
    if (!evt.event) return;
    if (!evt.event.startsWith("sse")) {
      return
    }
    let tryRecovery = false; 
    const type = evt.event
    const argsRawLines: Record<string, string[]> = {}

    const lines = evt.data.split('\n')
    for (const line of lines) {
      const colonIndex = line.indexOf(' ')
      const key = line.slice(0, colonIndex)
      let argLines = argsRawLines[key]
      if (!argLines) {
        argLines = []
        argsRawLines[key] = argLines
      }
      const value = line.slice(colonIndex + 1)
      argLines.push(value)
    }

    const argsRaw: Record<string, string> = {}
    for (const [key, lines] of Object.entries(argsRawLines)) {
      argsRaw[key] = lines.join('\n')
    }
    //console.log("event: ", argsRaw)
    switch (type) {
      case "sse-merge-alpine-data": {
        let {
          selector,
          data
        } = argsRaw;
        const targets = (selector ? document.querySelectorAll(selector) : [el]) as ElementWithXAttributes[];
        let json = JSON.parse(data) as { [key: string]: string };
        for (const target of targets) {
          let xdata = alpine.$data(target);
          alpine.effect(() => {
            for (const [key, value] of Object.entries(json)) {
              xdata[key] = value;
            }
          })
        }
        break
      }
      case "sse-execute-alpine-data": {
        let {
          selector,
          script
        } = argsRaw;
        const targets = (selector ? document.querySelectorAll(selector) : [el]) as ElementWithXAttributes[];
        for (const target of targets) {
          alpine.evaluate(target, script)
        }
        break;
      }
      case "sse-remove-fragments": {
        let {
          selector,
          useViewTransition:
          useViewTransitionRaw = `false`,
        } = argsRaw;
        const useViewTransition = isBoolString(useViewTransitionRaw)
        const removeTargets = document.querySelectorAll(selector)
        const applyToTargets = () => {
          for (const target of removeTargets) {
            target.remove()
          }
        }

        if (useViewTransition && supportsViewTransitions) {
          docWithViewTransitionAPI.startViewTransition(() => applyToTargets())
        } else {
          applyToTargets()
        }
        break;
      }
      case "sse-execute-script": {
        let {
          autoRemove: autoRemoveRaw = "true",
          attributes: attributesRaw = "type module",
          script,
        } = argsRaw;
        const autoRemove = isBoolString(autoRemoveRaw)
        const scriptEl = document.createElement('script')
        for (const attr of attributesRaw.split('\n')) {
          const pivot = attr.indexOf(' ')
          const key = pivot ? attr.slice(0, pivot) : attr
          const value = pivot ? attr.slice(pivot) : ''
          scriptEl.setAttribute(key.trim(), value.trim())
        }
        scriptEl.text = script
        document.head.appendChild(scriptEl)
        if (autoRemove) {
          scriptEl.remove()
        }
        break;
      }
      case "sse-merge-fragments": {
        let {
          fragments: fragmentsRaw = '<div></div>',
          selector = '',
          mergeMode = FragmentMergeModes.Morph,
          useViewTransition: useViewTransitionRaw = "false",
        } = argsRaw;
        const useViewTransition = isBoolString(useViewTransitionRaw)

        fragmentContainer.innerHTML = fragmentsRaw.trim()
        const fragments = [...fragmentContainer.content.children]
        for (const fragment of fragments) {
          const selectorOrID = selector || `#${fragment.getAttribute('id')}`
          const targets = [...(document.querySelectorAll(selectorOrID) || [])]
          if (!targets.length) {
            throw 'NoTargetsFound' //, ctx, { selectorOrID })
          }

          if (useViewTransition && supportsViewTransitions) {
            docWithViewTransitionAPI.startViewTransition(() =>
              applyToTargets(mergeMode, fragment, targets),
            )
          } else {
            applyToTargets(mergeMode, fragment, targets)
          }
        }
        break;
      }
      case "sse-error": {
        let {
          error = '',
        } = argsRaw;
        switch(error) {
          case "internal": {
            tryRecovery = true 
            break;
          }
        }
        break;
      }
    }
    el.dispatchEvent(
      new CustomEvent(type, {
        bubbles: true,
        composed: true,
        cancelable: true,
        detail: argsRaw
      }))
      if(tryRecovery) {
        throw "SseRemoteError" 
      }
  }

  const fragmentContainer = document.createElement('template')
  alpine.magic("sse", (el) => {
    function dispatchEvent(type: string, argsRaw: any) {
      el.dispatchEvent(
        new CustomEvent(type, {
          bubbles: true,
          composed: true,
          cancelable: true,
          detail: argsRaw
        }))
    }

    function sse(method: string) {
      const impl = async (url: string, args: SSEArgs) => {
        const {
          headers: userHeaders,
          contentType,
          selector,
          openWhenHidden,
          retryInterval,
          retryScaler,
          retryMaxWaitMs,
          retryMaxCount,
          abort,
          payload
        } = {
          ...{
            headers: {},
            contentType: 'json',
            selector: null,
            openWhenHidden: false, // will keep the request open even if the document is hidden.
            retryInterval: 2000, // the retry interval in milliseconds
            retryScaler: 2, // the amount to multiply the retry interval by each time
            retryMaxWaitMs: 30_000, // the maximum retry interval in milliseconds
            retryMaxCount: 10, // the maximum number of retries before giving up
            abort: undefined,
          },
          ...args
        }
        let cleanupFn = (): void => { }
        dispatchEvent(STARTED, {})
        try {
          if (!url?.length) {
            throw 'SseNoUrlProvided' //, ctx, { action })
          }
          const initialHeaders: Record<string, any> = {}
          initialHeaders["sse-request"] = true
          // We ignore the content-type header if using form data
          // if missing the boundary will be set automatically
          if (contentType === 'json') {
            initialHeaders['Content-Type'] = 'application/json'
          }

          const headers = Object.assign({}, initialHeaders, userHeaders)

          const req: FetchEventSourceInit = {
            method: method,
            headers,
            openWhenHidden,
            retryInterval,
            retryScaler,
            retryMaxWaitMs,
            retryMaxCount,
            signal: abort,
            onretryfailed: () => {
              dispatchEvent(RETRYING, {})
            },
            onopen: async (response: Response) => {
              if (response.status >= 400) {
                const status = response.status.toString()
                dispatchEvent(ERROR, { status })
              }
            },
            onmessage: (evt) => sse_dispatch_consumer(el,evt),
            onerror: (error) => {
              if (isWrongContent(error)) {
                // don't retry if the content-type is wrong
                throw 'InvalidContentType' //, ctx, { url })
              }
              // do nothing and it will retry
              if (error) {
                dispatchEvent(RETRYING, { message: error.message })
              }
            },
          }

          const urlInstance = new URL(url, window.location.origin)
          const queryParams = new URLSearchParams(urlInstance.search)
          if (contentType === 'json') {
            const json = JSON.stringify(payload || {})
            if (method === 'GET') {
              queryParams.set("sse", json)
            } else {
              req.body = json
            }
          } else if (contentType === 'form') {
            const formEl: HTMLFormElement | null = selector
              ? document.querySelector(selector)
              : el.closest('form')
            if (formEl === null) {
              if (selector) {
                throw 'SseFormNotFound' //, ctx, { action, selector })
              }
              throw 'SseClosestFormNotFound' //, ctx, { action })
            }
            if (el !== formEl) {
              const preventDefault = (evt: Event) => evt.preventDefault()
              formEl.addEventListener('submit', preventDefault)
              cleanupFn = (): void =>
                formEl.removeEventListener('submit', preventDefault)
            }
            if (!formEl.checkValidity()) {
              formEl.reportValidity()
              cleanupFn()
              return
            }
            const formData = new FormData(formEl)
            if (method === 'GET') {
              const formParams = new URLSearchParams(formData as any)
              for (const [key, value] of formParams) {
                queryParams.set(key, value)
              }
            } else {
              req.body = formData
            }
          } else {
            throw 'SseInvalidContentType' //, ctx, { action, contentType })
          }
          urlInstance.search = queryParams.toString()
          try {
            await fetchEventSource(urlInstance.toString(), req)
          } catch (error) {
            if (!isWrongContent(error)) {
              throw 'SseFetchFailed' //, ctx, { method, url, error })
            }
            // exit gracefully and do nothing if the content-type is wrong
            // this can happen if the client is sending a request
            // where no response is expected, and they haven't
            // set the content-type to text/event-stream
          }
        } finally {
          dispatchEvent(FINISHED, {})
          cleanupFn()
        }
      }
      return impl
    }
    return {
      get: sse('GET'),
      post: sse('POST'),
      patch: sse('PATCH'),
      put: sse('PUT'),
      delete: sse('DELETE')
    }
  })
 
  alpine.directive("sse",(el, {value, modifiers, expression}, {cleanup, evaluateLater, effect}) => {
      let evaluate = evaluateLater(expression)
      function dispatchEvent(type: string, argsRaw: any) {
        el.dispatchEvent(
          new CustomEvent(type, {
            bubbles: true,
            composed: true,
            cancelable: true,
            detail: argsRaw
          }))
      }
      const method = ['GET', 'POST', 'PATCH', 'PUT', 'DELETE'].find(k => modifiers.includes(k) ) || 'GET';

      let controller = new AbortController();
      let sse_url: string | undefined = undefined

      cleanup(() => {
        controller.abort()
      });
      function processInputValue(value: any) {
          const {
            headers: userHeaders,
            contentType,
            openWhenHidden,
            retryInterval,
            retryScaler,
            retryMaxWaitMs,
            retryMaxCount,
            payload,
            url
          } = {
            ...{
              headers: {},
              contentType: 'json',
              openWhenHidden: false, // will keep the request open even if the document is hidden.
              retryInterval: 2000, // the retry interval in milliseconds
              retryScaler: 2, // the amount to multiply the retry interval by each time
              retryMaxWaitMs: 30_000, // the maximum retry interval in milliseconds
              retryMaxCount: 10, // the maximum number of retries before giving up
              url: undefined
            },
            ...value
          }
          if(!url?.length || sse_url === url) {
            return;
          }
          sse_url = url;
          controller.abort()
          controller = new AbortController();
          dispatchEvent(STARTED, {})
          const initialHeaders: Record<string, any> = {}
          initialHeaders["sse-request"] = true
          // We ignore the content-type header if using form data
          // if missing the boundary will be set automatically
          if (contentType === 'json') {
            initialHeaders['Content-Type'] = 'application/json'
          }
          const headers = Object.assign({}, initialHeaders, userHeaders)
          const req: FetchEventSourceInit = {
            method: method,
            headers,
            openWhenHidden,
            retryInterval,
            retryScaler,
            retryMaxWaitMs,
            retryMaxCount,
            signal: controller.signal,
            onretryfailed: () => {
              dispatchEvent(RETRYING, {})
            },
            onopen: async (response: Response) => {
              if (response.status >= 400) {
                const status = response.status.toString()
                dispatchEvent(ERROR, { status })
              }
            },
            onmessage: (evt) => sse_dispatch_consumer(el,evt),
            onerror: (error) => {
              if (isWrongContent(error)) {
                // don't retry if the content-type is wrong
                throw 'InvalidContentType' //, ctx, { url })
              }
              // do nothing and it will retry
              if (error) {
                console.error(error.message)
                dispatchEvent(RETRYING, { message: error.message })
              }
            },
          }
          try {
            const urlInstance = new URL(url, window.location.origin)
            const queryParams = new URLSearchParams(urlInstance.search)
            if (contentType === 'json') {
              const json = JSON.stringify(payload || {})
              if (method === 'GET') {
                queryParams.set("sse", json)
              } else {
                req.body = json
              }
            } else {
              throw 'SseInvalidContentType' //, ctx, { action, contentType })
            }
            urlInstance.search = queryParams.toString()
            try {
              fetchEventSource(urlInstance.toString(), req)
            } catch (error) {
              if (!isWrongContent(error)) {
                throw 'SseFetchFailed' //, ctx, { method, url, error })
              }
              // exit gracefully and do nothing if the content-type is wrong
              // this can happen if the client is sending a request
              // where no response is expected, and they haven't
              // set the content-type to text/event-stream
            }
          }  finally {
            dispatchEvent(FINISHED, {})
          }
      }

      effect(() => {
        evaluate(async (value: any) => processInputValue(value))
      })
  })
}


