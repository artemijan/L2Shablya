use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use quick_xml::de::from_str;

#[derive(Debug, Serialize, Deserialize)]
struct SkillList {
    #[serde(rename = "skill", default)]
    skills: Vec<Skill>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Value {
    #[serde(rename = "@level")]
    level: Option<String>,
    #[serde(rename = "@fromLevel")]
    from_level: Option<String>,
    #[serde(rename = "@toLevel")]
    to_level: Option<String>,
    #[serde(rename = "@fromSubLevel")]
    from_sub_level: Option<String>,
    #[serde(rename = "@toSubLevel")]
    to_sub_level: Option<String>,
    #[serde(rename = "$value")]
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Parameter {
    Complex(ParameterStruct),
    Simple(String),
    Empty {},
}

#[derive(Debug, Serialize, Deserialize)]
struct ParameterStruct {
    #[serde(rename = "$value", default)]
    simple_value: Option<String>,
    #[serde(rename = "value", default)]
    values: Vec<Value>,
    #[serde(flatten, default)]
    children: std::collections::BTreeMap<String, Parameter>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Skill {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@toLevel")]
    to_level: String,
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@fromLevel")]
    from_level: Option<String>,
    #[serde(rename = "@displayId")]
    display_id: Option<String>,

    #[serde(flatten, default)]
    children: std::collections::BTreeMap<String, Parameter>,
}

pub fn run(input: &Path, output: &Path) -> Result<()> {
    if input.is_dir() {
        if !output.exists() {
            fs::create_dir_all(output).context("Failed to create output directory")?;
        }
        for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
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
    println!("Converting {:?} to {:?}", input, output);
    let content = fs::read_to_string(input).context(format!("Failed to read file {:?}", input))?;
    let skill_list: SkillList = from_str(&content).context(format!("Failed to parse XML from {:?}", input))?;
    let yaml = serde_yaml::to_string(&skill_list).context(format!("Failed to serialize to YAML for {:?}", input))?;
    fs::write(output, yaml).context(format!("Failed to write YAML to {:?}", output))?;
    Ok(())
}
