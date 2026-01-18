use color_eyre::owo_colors::OwoColorize;
use io::Error;
use std::{
    fs::{self, File},
    io::{self, ErrorKind},
};

use xmltree::{Element, ElementPredicate};

#[derive(Debug)]
pub struct MavenFile {
    root: Element,
    file_path: String,
}

impl MavenFile {
    pub fn from_file(file_path: String) -> Result<Self, std::io::Error> {
        let file_content: String = fs::read_to_string(&file_path)?;

        let xml_tree_root: Element = Element::parse(file_content.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .clone();

        return Ok(Self {
            root: xml_tree_root,
            file_path: file_path,
        });
    }

    pub fn get_dependencies(&self) -> Result<Vec<JavaDependency>, Error> {
        let mut dependencies_root = self.root.get_child("dependencies").ok_or(Error::new(
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

    pub fn update_dependencies(
        &mut self,
        updated_dependencies: &Vec<JavaDependency>,
    ) -> Result<(), Error> {
        let mut dependencies_root = self.root.get_mut_child("dependencies").ok_or(Error::new(
            ErrorKind::Other,
            "could not find dependencies element",
        ))?;

        dependencies_root.children.retain(|child| {
            child
                .as_element()
                .map(|element| {
                    let dependency = JavaDependency::from_element(element);

                    updated_dependencies
                        .iter()
                        .any(|elem| elem.group_id == dependency.group_id)
                })
                .unwrap_or(true)
        });

        self.update_xml_file();

        Ok(())
    }

    pub fn update_xml_file(&self) {
        &self.root.write(File::create("result.xml").unwrap());
    }
}

#[derive(Debug, PartialEq)]
pub struct JavaDependency {
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
