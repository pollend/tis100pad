use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct ManifestEntry {
    file: String,
    name: Option<String>,
    src: String,
    isEntry: Option<bool>,
    css: Option<Vec<String>>,
    imports: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
pub struct Manifest {
    #[serde(flatten)]
    inner: HashMap<String, ManifestEntry>,
}

impl Manifest {
    pub fn resolve_manifest_entry(&self, file: &str) -> Option<&ManifestEntry> {
        self.inner.get(file)
    }
    pub fn resolve_js<'a>(&'a self, file: &str) -> Vec<&'a str> {
        let mut result: Vec<&'a str> = Vec::new();
        self.__resolve_js(file, &mut result);
        result
    }

    fn __resolve_js<'a>(&'a self, file: &str, res: &mut Vec<&'a str>) {
        if let Some(manifest_entry) = self.inner.get(file) {
            if let Some(imports) = &manifest_entry.imports {
                for import in imports {
                    self.__resolve_js(import.as_ref(), res);
                }
            }
            res.push(manifest_entry.file.as_ref());
        }
    }

    pub fn resolve_css<'a>(&'a self, file: &str) -> Vec<&'a str> {
        let mut result: Vec<&'a str> = Vec::new();
        self.__resolve_css(file, &mut result);
        result
    }

    fn __resolve_css<'a>(&'a self, file: &str, res: &mut Vec<&'a str>) {
        if let Some(manifest_entry) = self.inner.get(file) {
            if let Some(imports) = &manifest_entry.imports {
                for import in imports {
                    self.__resolve_css(import.as_ref(), res);
                }
            }
            if let Some(css_manifest) = &manifest_entry.css {
                for css in css_manifest {
                    res.push(css.as_ref());
                }
            }
        }
    }
}

