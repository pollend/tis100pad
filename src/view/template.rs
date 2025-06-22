use askama::FastWritable;

use crate::{common::{manifest::Manifest, sse::Event}, view::sse_events::{SSEExecuteScriptBuilder, SSEMergeFragment}};


enum HeaderRoute {
    ProblemList,
    PadEditor,
    ProblemEditor,
}

pub struct DefaultBase<'a> {
    pub title: &'a str,
    pub base_uri: &'a str,
    pub manifest: &'a Manifest,
}

pub trait DefaultTemplate {
    fn __main_content(&self) -> impl FastWritable;
    fn __main_nav(&self) -> impl FastWritable;
    fn __root(&self) -> &DefaultBase<'_>;

    fn build_sse_main_content(&self) -> Result<Event, askama::Error> {
        SSEMergeFragment::new()
            .finish_askam(&self.__main_content())
    }

    fn build_sse_main_nav(&self) -> Result<Event, askama::Error> {
        SSEMergeFragment::new()
            .finish_askam(&self.__main_nav())
    }

    fn build_sse_title(&self) -> Event {
        SSEExecuteScriptBuilder::new().finish_title(self.__root().title)
    }

    fn manifest(&self) -> &Manifest {
        self.__root().manifest
    }

    fn base_uri(&self) -> &str {
        self.__root().base_uri
    }

    fn title(&self) -> &str {
        self.__root().title
    }
}
