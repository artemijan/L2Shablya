use anyhow::{Context, Result, bail};
use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::events::Event;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Element names that are always serialized as YAML sequences, even when a single
/// occurrence is present, so the strongly-typed loaders can rely on a stable shape.
const FORCE_LIST: [&str; 3] = ["skill", "value", "effect"];

#[derive(Debug, Default)]
struct Node {
    attrs: Vec<(String, String)>,
    text: String,
    children: Vec<(String, Node)>,
}

pub fn run(input: &Path, output: &Path) -> Result<()> {
    if input.is_dir() {
        if !output.exists() {
            fs::create_dir_all(output).context("Failed to create output directory")?;
        }
        for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "xml") {
                let relative_path = path.strip_prefix(input)?;
                let mut output_path = output.join(relative_path);
                output_path.set_extension("yaml");

                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                convert_file(path, &output_path)?;
            }
        }
    } else {
        let output_path = if output.is_dir() {
            let mut p = output.join(input.file_name().unwrap());
            p.set_extension("yaml");
            p
        } else {
            output.to_path_buf()
        };

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        convert_file(input, &output_path)?;
    }
    Ok(())
}

fn convert_file(input: &Path, output: &Path) -> Result<()> {
    println!("Converting {input:?} to {output:?}");
    let content = fs::read_to_string(input).context(format!("Failed to read file {input:?}"))?;
    let root = parse_xml(&content).context(format!("Failed to parse XML from {input:?}"))?;
    let yaml_value = node_to_yaml(&root);
    let yaml = serde_yaml::to_string(&yaml_value)
        .context(format!("Failed to serialize to YAML for {input:?}"))?;
    fs::write(output, yaml).context(format!("Failed to write YAML to {output:?}"))?;
    Ok(())
}

/// Parses the whole document and returns the root element as a [`Node`].
fn parse_xml(content: &str) -> Result<Node> {
    let mut reader = Reader::from_str(content);
    // A stack of (element name, node under construction); the bottom is a virtual root.
    let mut stack: Vec<(String, Node)> = vec![(String::new(), Node::default())];

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut node = Node::default();
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = attr.normalized_value(XmlVersion::Implicit1_0)?.to_string();
                    node.attrs.push((key, val));
                }
                stack.push((name, node));
            }
            Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut node = Node::default();
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = attr.normalized_value(XmlVersion::Implicit1_0)?.to_string();
                    node.attrs.push((key, val));
                }
                let parent = &mut stack.last_mut().unwrap().1;
                parent.children.push((name, node));
            }
            Event::End(_) => {
                let (name, node) = stack.pop().unwrap();
                if stack.is_empty() {
                    bail!("Unbalanced XML: unexpected closing tag </{name}>");
                }
                let parent = &mut stack.last_mut().unwrap().1;
                parent.children.push((name, node));
            }
            Event::Text(e) => {
                let text = e.xml10_content()?.to_string();
                let node = &mut stack.last_mut().unwrap().1;
                node.text.push_str(&text);
            }
            Event::CData(e) => {
                let text = String::from_utf8_lossy(&e).to_string();
                let node = &mut stack.last_mut().unwrap().1;
                node.text.push_str(&text);
            }
            Event::Eof => break,
            _ => {} // comments, processing instructions, declarations
        }
    }

    if stack.len() != 1 {
        bail!("Unbalanced XML: {} unclosed elements", stack.len() - 1);
    }
    let (_, virtual_root) = stack.pop().unwrap();
    // The document has exactly one root element (e.g. <list>); return it.
    virtual_root
        .children
        .into_iter()
        .map(|(_, node)| node)
        .next()
        .context("Empty XML document")
}

/// Converts a [`Node`] to a YAML value:
/// - attributes become `'@name'` keys,
/// - trimmed text content becomes a `$text` key,
/// - child elements become nested mappings, repeated (or force-listed) names become sequences.
fn node_to_yaml(node: &Node) -> Value {
    let mut map = Mapping::new();
    for (key, val) in &node.attrs {
        map.insert(Value::String(format!("@{key}")), Value::String(val.clone()));
    }
    let text = node.text.trim();
    if !text.is_empty() {
        map.insert(
            Value::String("$text".to_string()),
            Value::String(text.to_string()),
        );
    }

    // Group children by name, preserving the order of first appearance.
    let mut order: Vec<&str> = Vec::new();
    for (name, _) in &node.children {
        if !order.contains(&name.as_str()) {
            order.push(name);
        }
    }
    for name in order {
        let values: Vec<Value> = node
            .children
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, child)| node_to_yaml(child))
            .collect();
        let yaml_val = if values.len() > 1 || FORCE_LIST.contains(&name) {
            Value::Sequence(values)
        } else {
            values.into_iter().next().unwrap()
        };
        map.insert(Value::String(name.to_string()), yaml_val);
    }
    Value::Mapping(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_level_values_are_preserved() {
        let xml = r#"<list><skill id="1177" toLevel="5" name="Wind Strike">
        <castRange>600</castRange>
        <effectPoint>
            <value level="1">-92</value>
            <value level="2">-106</value>
        </effectPoint>
        <targetType>
            <value level="1">ENEMY_ONLY</value>
            <value level="2">ENEMY</value>
        </targetType>
        <effects><effect name="MagicalAttack"><power><value fromLevel="1" toLevel="2">12</value></power></effect></effects>
        </skill></list>"#;
        let root = parse_xml(xml).unwrap();
        let yaml = node_to_yaml(&root);

        let skill = &yaml["skill"][0];
        assert_eq!(skill["@id"], Value::String("1177".into()));
        assert_eq!(skill["castRange"]["$text"], Value::String("600".into()));
        assert_eq!(
            skill["effectPoint"]["value"][0]["@level"],
            Value::String("1".into())
        );
        assert_eq!(
            skill["effectPoint"]["value"][1]["$text"],
            Value::String("-106".into())
        );
        assert_eq!(
            skill["targetType"]["value"][0]["$text"],
            Value::String("ENEMY_ONLY".into())
        );
        let effect = &skill["effects"]["effect"][0];
        assert_eq!(effect["@name"], Value::String("MagicalAttack".into()));
        assert_eq!(
            effect["power"]["value"][0]["@fromLevel"],
            Value::String("1".into())
        );
        assert_eq!(
            effect["power"]["value"][0]["$text"],
            Value::String("12".into())
        );
    }
}
