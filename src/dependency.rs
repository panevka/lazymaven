use io::Error;
use std::{
    fs,
    io::{self, ErrorKind},
};

use xmltree::{Element, ElementPredicate};

#[derive(Debug)]
pub struct MavenFile {
    root: Element,
}

impl MavenFile {
    pub fn from_file(file_path: String) -> Result<Self, std::io::Error> {
        let file_content: String = fs::read_to_string(file_path)?;

        let xml_tree_root: Element = Element::parse(file_content.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .clone();

        return Ok(Self {
            root: xml_tree_root,
        });
    }

    pub fn get_dependencies(&self) -> Result<Vec<JavaDependency>, Error> {
        let dependencies_root = self.root.get_child("dependencies").ok_or(Error::new(
            ErrorKind::Other,
            "could not find dependencies element",
        ))?;

        let dependencies: Vec<JavaDependency> = dependencies_root
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .map(|dependency| JavaDependency::from_element(dependency))
            .collect();

        Ok(dependencies)
    }

    pub fn remove_dependency(&mut self, index: usize) {
        if let Some(dependencies) = self.root.get_mut_child("dependencies") {
            dependencies.children.remove(index);
        }
    }
}

#[derive(Debug, PartialEq)]
struct JavaDependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

impl<'a> JavaDependency {
    fn from_element(dependency: &'a Element) -> JavaDependency {
        let mut group_id = None;
        let mut artifact_id = None;
        let mut version = None;

        for field in dependency.children.iter().filter_map(|f| f.as_element()) {
            let value = field.children[0].as_text().unwrap_or("").to_string();
            match field.name.as_str() {
                "groupId" => group_id = Some(value),
                "artifactId" => artifact_id = Some(value),
                "version" => version = Some(value),
                _ => {}
            }
        }

        JavaDependency {
            group_id: group_id.unwrap_or_default(),
            artifact_id: artifact_id.unwrap_or_default(),
            version: version.unwrap_or_default(),
        }
    }

    fn edit_dependency_version(&mut self, dependency_version: String) {
        self.version = dependency_version;
    }
}

impl ElementPredicate for JavaDependency {
    fn match_element(&self, e: &Element) -> bool {
        let dependency = JavaDependency::from_element(e);
        dependency == *self
    }
}
impl ElementPredicate for &JavaDependency {
    fn match_element(&self, e: &Element) -> bool {
        let dependency = JavaDependency::from_element(e);
        dependency == **self
    }
}
