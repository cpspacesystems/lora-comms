use std::{env, error::Error, fs, path::Path, str::Lines, time};

use crate::codegen::{DataDefEntry, Direction, NetworkType, PollRate, generator::Generator};
type AnyError = Box<dyn Error + Send + Sync>;

const DOC_TYPE: &str = "!LORA_DATA_DEF-V0.1";
const SCHEMA_FORMAT: [[&str; 2]; 5] = [
    ["poll-rate", "enum"],
    ["size", "u64"],
    ["network-type", "enum"],
    ["source-path", "string-utf8"],
    ["dest-path", "string-utf8"]
];
const SCHEMA_SECTIONS: &[&str] = &["RocketToGround", "GroundToRocket"];
const SCHEMA_ENUMS: &[(&str, &[&str])] = &[
    ("poll-rate", &["1Hz", "0.5Hz", "10Hz", "OnChange", "ASAP"]),
    ("network-type", &["tism", "zenoh"]),
];
const COMMENT: &str = "//";
const ESCAPED_COMMENT: &str = "\\//";

fn map_poll_rate(rate: impl AsRef<str>) -> PollRate {
    match rate.as_ref() {
        "OnChange" => PollRate::OnChange,
        "ASAP" => PollRate::ASAP,
        s if s.ends_with("Hz") => PollRate::FixedRate(
            time::Duration::from_secs_f64(s
                [..s.len()-2].parse().expect("DATA DEF PARSING: Expected a valid number for poll-rate")
            )
        ), 
        _ => panic!("DATA DEF PARSING: Unregonized poll-rate.")
    }
}

fn map_network_type(net_type: impl AsRef<str>) -> NetworkType {
    match net_type.as_ref() {
        "tism" => NetworkType::TISM,
        "zenoh" => NetworkType::Zenoh,
        _ => panic!("DATA DEF PARSING: Unregonized network-type")
    }
}

struct DataParserContext {
    ground_to_rocket: Generator,
    rocket_to_ground: Generator,
}
impl DataParserContext {
    pub fn new() -> Self {
        DataParserContext { ground_to_rocket: Generator::new(Direction::GroundToRocket), rocket_to_ground: Generator::new(Direction::RocketToGround) }
    }
}

fn line_to_entry(line_ref: impl AsRef<str>) -> DataDefEntry {
    let line = line_ref.as_ref();
    let mut items = line.split(',').map(|s| s.trim());
    
    DataDefEntry {
        rate: map_poll_rate(items.next().expect("DATA DEF PARSING: Expected poll-rate")),
        size: items.next().expect("DATA DEF PARSING: Expected size").parse().expect("DATA DEF PARSING: Expected valid integer"),
        network: map_network_type(items.next().expect("DATA DEF PARSING: Expected network-type")),
        source: items.next().expect("DATA DEF PARSING: Expected source-path").to_owned(),
        destination: items.next().expect("DATA DEF PARSING: Expected dest-path").to_owned(),
    }
}

fn parse_rocket_to_ground<'a>(ctx: &mut DataParserContext, lines: &mut impl Iterator<Item = &'a str>) {
    loop {
        let line = get_line(lines);
    
        if line.starts_with("#") {
            let field: Vec<&str> = line.split_whitespace().collect();
            assert!(field.len() == 2 && field[0] == "#end" && field[1] == "RocketToGround",
                "DATA DEF PARSING: Expected end of RocketToGround, but got {}", line);
            return;
        } else {
            let entry = line_to_entry(line);
            ctx.rocket_to_ground.add_entry_producing(&entry);
            ctx.ground_to_rocket.add_entry_consuming(&entry);
        }        
    }
}

fn parse_ground_to_rocket<'a>(ctx: &mut DataParserContext, lines: &mut impl Iterator<Item = &'a str>) {
    loop {
        let line = get_line(lines);
    
        if line.starts_with("#") {
            let field: Vec<&str> = line.split_whitespace().collect();
            assert!(field.len() == 2 && field[0] == "#end" && field[1] == "GroundToRocket",
                "DATA DEF PARSING: Expected end of GroundToRocket, but got {}", line);
            return;
        } else {
            let entry = line_to_entry(line);
            ctx.ground_to_rocket.add_entry_producing(&entry);
            ctx.rocket_to_ground.add_entry_consuming(&entry);
        }
    }
}

fn defined_sections<'a>(section: &str, lines: &mut impl Iterator<Item = &'a str>, ctx: &mut DataParserContext) {
    match section {
        "schema" => verify_schema(lines),
        "RocketToGround" => parse_rocket_to_ground(ctx, lines),
        "GroundToRocket" => parse_ground_to_rocket(ctx, lines),
        s => panic!("DATA DEF PARSING: {} has no section {} defined!", DOC_TYPE, s)
    }
}

pub fn parse_data_def(path: &str) {
    let out_dir = env::var_os("OUT_DIR").expect("DATA DEF PARSING: OUT_DIR not set");

    // open file
    let data = fs::read_to_string(path).unwrap_or_else(|e| panic!("Unable to open {} with error: {}", path, e));
    let mut lines = data.lines()
        .map(|s| s.trim()) // trim whitespace
        .filter(|s| !s.is_empty()) // filter out empty lines
        .filter(|s| !s.starts_with(COMMENT)) // filter out line comments
    ;

    // veryify doctype
    verify_doc_type(&mut lines);
    
    let mut ctx = DataParserContext::new();

    // parse sections 
    while let Some(l) = lines.next() {
        let section = parse_section_begin(l);
        defined_sections(section.as_str(), &mut lines, &mut ctx);
    }

    // finalize and output generated files
    fs::write(Path::new(&out_dir).join("codegen_rocket_to_ground.rs"), ctx.rocket_to_ground.finalize())
        .expect("DATA DEF PARSING: Failed to write codegen_rocket_to_ground.rs!");
    fs::write(Path::new(&out_dir).join("codegen_ground_to_rocket.rs"), ctx.ground_to_rocket.finalize())
        .expect("DATA DEF PARSING: Failed to write codegen_ground_to_rocket.rs!");
}

fn verify_doc_type<'a>(lines: &mut impl Iterator<Item = &'a str>) {
    let line = get_line(lines);
    let fields: Vec<&str> = line.split_whitespace().collect();

    assert!(fields.len() == 2, "DATA DEF PARSING: Expected 2 fields for doctype, but got {}", fields.len());
    assert!(fields[0] == "#doctype", "DATA DEF PARSING: Expected doctype header, got {}", fields[0]);
    assert!(fields[1] == DOC_TYPE, "DATA DEF PARSING: Build script is on version {}, but data defination is on version {}", DOC_TYPE, fields[1]);
}

fn verify_schema<'a>(lines: &mut impl Iterator<Item = &'a str>) {
    loop { 
        let (line_type, contents) = parse_line_components(lines);
        match line_type.as_str() {
        "format" => {
            let mut fields = contents.split(',')
                .map(|s| s.trim())
                .map(|s| {
                    assert!(s.len() > 3 && s.starts_with('<') && s.ends_with('>'), "DATA DEF PARSING: Format Malformed field {}", s);
                    &s[1..s.len()-1]
                })
                .map(|s| 
                    if let Some((name, dtype)) = s.split_once(':') {
                        [name.trim(), dtype.trim()]
                    } else {
                        panic!("DATA DEF PARSING: Format malformed field, missing `:`: {}", s);
                    }
                )
            ;
            let mut expected_iter = SCHEMA_FORMAT.iter();
            loop {match (fields.next(), expected_iter.next()) {
                (Some(f), Some(e)) if f == *e => continue,
                (Some(f), Some(e)) => panic!("DATA DEF PARSING: Format expected field <{}:{}>, got <{}:{}>", e[0], e[1], f[0], f[1]),
                (None, None) => break,
                _ => panic!("DATA DEF PARSING: Format: expected number of fields does not match up with number of fields defined in schema!")
            }}
        },
        "section" => {
            let section = contents.trim();
            assert!(SCHEMA_SECTIONS.contains(&section), "DATA DEF PARSING: Section {} is not any of the expected sectinons: {:?}", section, SCHEMA_SECTIONS);
        },
        "enum" => {
            let mut fields = contents.split_whitespace()
                .map(|x| x.trim())
                .filter(|x| !x.is_empty());
            let name = fields.next().expect("DATA DEF PARSING: Expected enum name, but got nothing.");
            
            let expected_enum = SCHEMA_ENUMS.iter().find(|e| e.0 == name).expect(format!("DATA DEF PARSING: {} has no enum {} defined!", DOC_TYPE, name).as_str());
            let current_items: Vec::<&str> = fields.collect();
            assert!(expected_enum.1.len() == current_items.len(), "DATA DEF PARSING: Expected {} items for enum {}, got {} items.", expected_enum.1.len(), expected_enum.0, current_items.len());
            assert!(expected_enum.1.iter().all(|e| current_items.contains(e)),
                "DATA DEF PARSING: Expected enum {0} to contain items {1:?}. But got a schema with enum {0} containg items {2:?}",
                expected_enum.0, expected_enum.1, current_items
            );
        },
        "end" => {
            assert!(contents.trim() == "schema", "DATA DEF PARSING: Expected end of schema, but got {}", contents);
            return;
        },
        l => panic!("DATA DEF PARSING: Invalid schema item: {}", l)
    }}
}

fn parse_section_begin(line: &str) -> String  {
    let line = get_line_with_line(line);
    let fields: Vec<&str> = line.split_whitespace().collect();

    assert!(fields.len() == 2, "DATA DEF PARSING: Expected 2 fields for section begin, but got {}", fields.len());
    assert!(fields[0] == "#begin", "DATA DEF PARSING: Expected #begin, got {}", fields[0]);
    
    fields[1].to_owned()
}

fn get_line<'a>(lines: &mut impl Iterator<Item = &'a str>) -> String {
    let line = lines.next().expect("DATA DEF PARSING: Expected line!");
    
    get_line_with_line(line)
}

fn get_line_with_line(line: &str) -> String {
    // split out comments:
    let mut res = Vec::new();

    let mut last_slice_end = 0;
    let window_size = ESCAPED_COMMENT.len();
    let window_comment_start = window_size - COMMENT.len();
    let mut window_begin = 0;
    let mut window_end = window_begin + window_size;
    while window_end < line.len() {
        let window = &line[window_begin..window_end];
        if window == ESCAPED_COMMENT {
            // push the previous and decoded comment if we hit an escaped comment
            res.push(&line[last_slice_end..window_begin]);
            res.push(COMMENT);
            last_slice_end = window_end;
        } else if &window[window_comment_start..] == COMMENT { 
            // push the rest and exit if we hit a comment
            res.push(&line[last_slice_end..window_begin + window_comment_start]);
            break;
        }
        window_begin = window_end;
        window_end = window_begin + window_size;
    }
    
    // no comment
    if res.len() == 0 {
        line.to_owned()
    // a singluar comment with no escapes
    } else if res.len() == 1{
        res[0].to_owned()
    // mutiple components
    } else {
        res.join("")
    }
}

fn parse_line_components<'a>(lines: &mut impl Iterator<Item = &'a str>) -> (String, String) {
    let line = get_line(lines);
    if let Some((line_type, line_content)) = line.split_once(char::is_whitespace) {
        assert!(line_type.starts_with('#'), "DATA DEF PARSING: Expected line starting with #, but got {}", line_type);
        (line_type[1..].to_owned(), line_content.to_owned())
    } else {
        panic!("DATA DEF PARSING: Expected line starting with #");
    }
}


