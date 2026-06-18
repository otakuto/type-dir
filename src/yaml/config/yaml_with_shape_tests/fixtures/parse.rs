use crate::yaml::config::YamlWithShape;

pub fn parse(yaml: &str) -> YamlWithShape {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).expect("yaml parse failed");
    YamlWithShape(value)
}
