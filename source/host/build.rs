use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct FieldDef {
    number: u64,
    is_map: bool,
    child_symbol: Option<String>,
    child_type: Option<String>,
    map_message: bool,
}

#[derive(Debug, Clone, Default)]
struct MessageDef {
    fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
struct BlobField {
    proto_field_name: String,
    field_number: u64,
    blob_reference_type: String,
}

fn quoted_after(input: &str, marker: &str) -> Option<String> {
    let rest = input.get(input.find(marker)? + marker.len()..)?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn number_after(input: &str, marker: &str) -> Option<u64> {
    let rest = input.get(input.find(marker)? + marker.len()..)?.trim_start();
    let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

fn identifier_after(input: &str, marker: &str) -> Option<String> {
    let rest = input.get(input.find(marker)? + marker.len()..)?.trim_start();
    let ident: String = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect();
    (!ident.is_empty()).then_some(ident)
}

fn split_top_level_objects(input: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start.take() {
                        objects.push(&input[start..=index]);
                    }
                }
            }
            _ => {}
        }
    }
    objects
}

fn parse_message_definitions(
    content: &str,
    defs: &mut BTreeMap<String, MessageDef>,
    symbol_to_type: &mut BTreeMap<String, String>,
) {
    let type_marker = ".typeName = \"agent.v1.";
    let fields_marker = ").fields = proto3.util.newFieldList(() => [";
    let mut cursor = 0usize;
    while let Some(relative) = content[cursor..].find(type_marker) {
        let type_pos = cursor + relative;
        let line_start = content[..type_pos].rfind('\n').map(|idx| idx + 1).unwrap_or(0);
        let line_end = content[type_pos..]
            .find('\n')
            .map(|idx| type_pos + idx)
            .unwrap_or(content.len());
        let line = &content[line_start..line_end];
        let symbol = line
            .trim_start()
            .strip_prefix('(')
            .and_then(|value| value.split(" as MutableMessageType").next())
            .expect("generated message typeName line must expose a symbol")
            .trim()
            .to_string();
        let type_name = quoted_after(line, ".typeName = ")
            .expect("generated message typeName must be quoted");
        if let Some(previous) = symbol_to_type.insert(symbol.clone(), type_name.clone()) {
            assert_eq!(
                previous, type_name,
                "generated protobuf symbol {symbol} mapped to multiple type names"
            );
        }

        let after = &content[line_end..];
        let symbol_marker = format!("({symbol} as MutableMessageType<");
        let symbol_fields = after
            .find(&symbol_marker)
            .expect("generated message fields must follow typeName");
        let fields_start = line_end + symbol_fields;
        let marker_pos = content[fields_start..]
            .find(fields_marker)
            .expect("generated message field list marker");
        let array_start = fields_start + marker_pos + fields_marker.len();
        let array_end = content[array_start..]
            .find("]);")
            .map(|idx| array_start + idx)
            .expect("generated message field list terminator");
        let array = &content[array_start..array_end];

        let mut fields = Vec::new();
        for object in split_top_level_objects(array) {
            let Some(number) = number_after(object, "no:") else {
                continue;
            };
            let kind = quoted_after(object, "kind:").unwrap_or_default();
            if kind == "message" {
                fields.push(FieldDef {
                    number,
                    is_map: false,
                    child_symbol: identifier_after(object, "T:"),
                    child_type: None,
                    map_message: false,
                });
            } else if kind == "map" {
                let map_value = object
                    .find("V:")
                    .and_then(|idx| object.get(idx + 2..))
                    .unwrap_or("");
                let map_message = quoted_after(map_value, "kind:")
                    .is_some_and(|value| value == "message");
                fields.push(FieldDef {
                    number,
                    is_map: true,
                    child_symbol: map_message.then(|| identifier_after(map_value, "T:")).flatten(),
                    child_type: None,
                    map_message,
                });
            } else {
                fields.push(FieldDef {
                    number,
                    is_map: false,
                    child_symbol: None,
                    child_type: None,
                    map_message: false,
                });
            }
        }
        defs.insert(type_name, MessageDef { fields });
        cursor = array_end + 3;
    }
}

fn parse_blob_metadata(content: &str) -> BTreeMap<String, Vec<BlobField>> {
    let section_end = content
        .find("export function getBlobReferenceMessageMetadata")
        .unwrap_or(content.len());
    let mut current_type: Option<String> = None;
    let mut metadata = BTreeMap::<String, Vec<BlobField>>::new();

    for line in content[..section_end].lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("\"agent.v1.") && trimmed.contains("\": {") {
            current_type = trimmed
                .strip_prefix('"')
                .and_then(|value| value.split("\": {").next())
                .map(str::to_string);
        }

        let Some(source_type) = current_type.as_ref() else {
            continue;
        };
        let mut remainder = line;
        while let Some(index) = remainder.find("{ protoFieldName:") {
            remainder = &remainder[index + 2..];
            let object_end = remainder.find('}').unwrap_or(remainder.len());
            let object = &remainder[..object_end];
            let Some(proto_field_name) = quoted_after(object, "protoFieldName:") else {
                break;
            };
            let Some(field_number) = number_after(object, "fieldNumber:") else {
                break;
            };
            let Some(blob_reference_type) = quoted_after(object, "blobReferenceType:") else {
                break;
            };
            metadata
                .entry(source_type.clone())
                .or_default()
                .push(BlobField {
                    proto_field_name,
                    field_number,
                    blob_reference_type,
                });
            remainder = &remainder[object_end..];
        }
    }
    metadata
}

fn rust_string(value: &str) -> String {
    format!("{value:?}")
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let proto_dir = manifest_dir.join("../packages/proto/generated/agent/v1");
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    let mut defs = BTreeMap::<String, MessageDef>::new();
    let mut symbol_to_type = BTreeMap::<String, String>::new();

    let mut files = fs::read_dir(&proto_dir)
        .expect("read generated protobuf directory")
        .map(|entry| entry.expect("read generated protobuf entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("_pb.ts"))
        })
        .collect::<Vec<_>>();
    files.sort();

    for path in &files {
        let content = fs::read_to_string(path).expect("read generated protobuf TypeScript");
        parse_message_definitions(&content, &mut defs, &mut symbol_to_type);
    }

    for def in defs.values_mut() {
        for field in &mut def.fields {
            field.child_type = field
                .child_symbol
                .as_ref()
                .and_then(|symbol| symbol_to_type.get(symbol))
                .cloned();
        }
    }

    let metadata_path = proto_dir.join("blob-reference-metadata.ts");
    let metadata_content =
        fs::read_to_string(&metadata_path).expect("read blob-reference-metadata.ts");
    let metadata = parse_blob_metadata(&metadata_content);

    let mut proto_blob_types = BTreeSet::<String>::new();
    for fields in metadata.values() {
        for field in fields {
            if field.blob_reference_type.starts_with("agent.v1.") {
                proto_blob_types.insert(field.blob_reference_type.clone());
            }
        }
    }
    for type_name in &proto_blob_types {
        assert!(
            defs.contains_key(type_name),
            "blob-reference metadata target {type_name} has no generated message definition"
        );
    }

    let mut reachable = BTreeSet::<String>::new();
    let mut queue = VecDeque::<String>::new();
    queue.push_back("agent.v1.ConversationStateStructure".to_string());
    for type_name in &proto_blob_types {
        queue.push_back(type_name.clone());
    }
    while let Some(type_name) = queue.pop_front() {
        if !reachable.insert(type_name.clone()) {
            continue;
        }
        let Some(def) = defs.get(&type_name) else {
            continue;
        };
        for field in &def.fields {
            if let Some(child_type) = field.child_type.as_ref() {
                if child_type.starts_with("agent.v1.") && !reachable.contains(child_type) {
                    queue.push_back(child_type.clone());
                }
            }
        }
    }

    let mut output = String::new();
    output.push_str("// @generated by source/host/build.rs from frozen Grok protobuf metadata.\n");
    output.push_str("// Do not edit by hand; change the generated TypeScript metadata/schema inputs instead.\n\n");
    output.push_str("#[derive(Debug, Clone, Copy)]\n");
    output.push_str("struct BlobReferenceFieldDescriptor { field_number: u64, blob_reference_type: &'static str, map_value: bool, skip: bool }\n");
    output.push_str("#[derive(Debug, Clone, Copy)]\n");
    output.push_str("struct NestedMessageFieldDescriptor { field_number: u64, child_type: &'static str, map_value: bool }\n");
    output.push_str("#[derive(Debug, Clone, Copy)]\n");
    output.push_str("struct MessageDescriptor { blob_fields: &'static [BlobReferenceFieldDescriptor], nested_fields: &'static [NestedMessageFieldDescriptor] }\n\n");

    output.push_str("fn is_proto_blob_reference_type(type_name: &str) -> bool {\n    matches!(type_name,\n");
    for type_name in &proto_blob_types {
        output.push_str("        ");
        output.push_str(&rust_string(type_name));
        output.push_str(" |\n");
    }
    output.push_str("        \"__never__\"\n    )\n}\n\n");

    output.push_str("fn message_descriptor(type_name: &str) -> Option<MessageDescriptor> {\n    match type_name {\n");
    for type_name in &reachable {
        let Some(def) = defs.get(type_name) else {
            continue;
        };
        output.push_str("        ");
        output.push_str(&rust_string(type_name));
        output.push_str(" => Some(MessageDescriptor {\n            blob_fields: &[\n");
        if let Some(blob_fields) = metadata.get(type_name) {
            for blob in blob_fields {
                let map_value = def
                    .fields
                    .iter()
                    .find(|field| field.number == blob.field_number)
                    .is_some_and(|field| field.is_map);
                let edge_name = format!("{type_name}.{}", blob.proto_field_name);
                let skip = matches!(
                    edge_name.as_str(),
                    "agent.v1.UserMessage.conversation_state_blob_id"
                        | "agent.v1.ConversationStateStructure.summary_archive"
                        | "agent.v1.ConversationStateStructure.summary_archives"
                );
                output.push_str("                BlobReferenceFieldDescriptor { field_number: ");
                output.push_str(&blob.field_number.to_string());
                output.push_str(", blob_reference_type: ");
                output.push_str(&rust_string(&blob.blob_reference_type));
                output.push_str(", map_value: ");
                output.push_str(if map_value { "true" } else { "false" });
                output.push_str(", skip: ");
                output.push_str(if skip { "true" } else { "false" });
                output.push_str(" },\n");
            }
        }
        output.push_str("            ],\n            nested_fields: &[\n");
        for field in &def.fields {
            if !field.map_message && field.child_symbol.is_none() {
                continue;
            }
            let Some(child_type) = field.child_type.as_ref() else {
                continue;
            };
            if !reachable.contains(child_type) {
                continue;
            }
            output.push_str("                NestedMessageFieldDescriptor { field_number: ");
            output.push_str(&field.number.to_string());
            output.push_str(", child_type: ");
            output.push_str(&rust_string(child_type));
            output.push_str(", map_value: ");
            output.push_str(if field.is_map { "true" } else { "false" });
            output.push_str(" },\n");
        }
        output.push_str("            ],\n        }),\n");
    }
    output.push_str("        _ => None,\n    }\n}\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::write(out_dir.join("conversation_blob_gc_metadata.rs"), output)
        .expect("write conversation blob GC metadata");
}
