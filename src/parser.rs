use std::{collections::HashMap, fmt, fs, io, iter::Map};

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

    pub fn get_dependencies(&self) -> Vec<HashMap<String, String>> {
        let mut dependencies: Vec<HashMap<String, String>> = Vec::new();

        let deps_root = match self.root.get_child("dependencies") {
            Some(d) => d,
            None => return dependencies,
        };

        for dependency in &deps_root.children {
            let mut dependency_map: HashMap<String, String> = HashMap::new();

            if let Some(dependency_fields) = dependency.as_element() {
                for field in &dependency_fields.children {
                    let field_element = field.as_element();

                    if let Some(element) = field_element {
                        let field_name = &element.name;
                        let field_value = element.children[0].as_text();

                        dependency_map.insert(
                            field_name.clone(),
                            field_value.unwrap_or("NULL").to_string(),
                        );
                    }
                }
            }

            dependencies.push(dependency_map);
        }

        return dependencies;
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
