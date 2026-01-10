use io::{Error, ErrorKind::Other};
use std::{
    fmt, fs,
    io::{self, ErrorKind},
};

use xmltree::Element;

pub struct MavenFile {
    root: Element,
}

impl MavenFile {
    pub fn from_file(file_path: String) -> Result<Self, std::io::Error> {
        let file_content: String = fs::read_to_string(file_path)?;
        let xml_tree_root = Element::parse(file_content.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        return Ok(Self {
            root: xml_tree_root,
        });
    }

    pub fn get_dependencies(&self) -> Result<Vec<JavaDependencyNode>, io::Error> {
        let dependencies_root = self.root.get_child("dependencies").ok_or(Error::new(
            ErrorKind::Other,
            "could not find dependencies element",
        ))?;

        let dependencies: Vec<JavaDependencyNode> = dependencies_root
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .map(|dependency| {
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

                let node = JavaDependencyNode {
                    group_id: group_id.unwrap_or_default(),
                    artifact_id: artifact_id.unwrap_or_default(),
                    version: version.unwrap_or_default(),
                };

                node
            })
            .collect();

        Ok(dependencies)
    }
}

impl fmt::Debug for MavenFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MavenFile")
            .field("root", &self.root)
            .finish()
    }
}

impl Default for MavenFile {
    fn default() -> Self {
        Self {
            root: Element::new("default"),
        }
    }
}

struct JavaDependencies {
    node: Element,
    dependencies: Vec<JavaDependencyNode>,
}

#[derive(Debug, Default)]
pub struct JavaDependencyNode {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}
