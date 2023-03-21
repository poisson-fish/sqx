use std::{collections::HashMap, iter};

fn normalize_headers(header: String) -> String {
    header
        .to_lowercase()
        .replace("local address", "local_address")
        .replace("foreign address", "foreign_address")
        .replace("pid/program name", "program_name")
        .replace("security context", "security_context")
        .replace("i-node", " inode")
        .replace("-", "_")
}

fn normalize_route_headers(header: String) -> String {
    header
        .to_lowercase()
        .replace("flags", "route_flags")
        .replace("ref", "route_refs")
        .replace("-", "_")
}

fn normalize_interface_headers(header: String) -> String {
    header.to_lowercase().replace("-", "_")
}

fn parse_network(headers: &Vec<String>, entry: &str) -> HashMap<String, String> {
    // Count words in header
    // if len of line is one less than len of header, then insert None in field 5
    let mut entry = entry.splitn(headers.len() - 1, ' ').collect::<Vec<&str>>();
    if entry.len() == headers.len() - 1 {
        entry.insert(5, "None");
    }

    let output_line: Vec<_> = headers
        .iter()
        .zip(entry.iter())
        .map(|(x, y)| (x.clone(), y.to_string()))
        .collect();

    let mut output_line_map = HashMap::new();
    for (key, value) in output_line {
        output_line_map.insert(key, value);
    }
    output_line_map.insert(String::from("kind"), String::from("network"));

    return output_line_map;
}

fn parse_socket(header_text: &str, headers: &Vec<String>, entry: &str) -> HashMap<String, String> {
    // get the column # of first letter of "state"
    let state_col = header_text.find("state").unwrap();

    // get the program name column area
    let pn_start = header_text.find("program_name").unwrap();
    let pn_end = header_text.find("path").unwrap() - 1;

    // remove [ and ] from each line
    let mut entry = entry.replace("[ ]", "---");
    entry = entry.replace("[", " ").replace("]", " ");

    // find program_name column area and substitute spaces with \u2063 there
    let old_pn = &entry[pn_start..pn_end];
    let new_pn = old_pn.replace(' ', "\u{2063}");
    entry = entry.replacen(old_pn, &new_pn, 1);

    let entry_list: Vec<&str> = entry.splitn(headers.len(), char::is_whitespace).collect();

    // check column # to see if state column is populated
    let mut output_line = HashMap::new();
    for i in 0..headers.len() {
        let val = if i == state_col && entry_list[i].chars().all(char::is_whitespace) {
            None
        } else {
            Some(entry_list[i].to_string())
        };
        output_line.insert(headers[i].clone(), val.unwrap_or_default());
    }

    output_line.insert("kind".to_string(), "socket".to_string());

    // fix program_name field to turn \u2063 back to spaces
    if let Some(program_name) = output_line.get_mut("program_name") {
        if !program_name.is_empty() {
            let old_d_pn = program_name.clone();
            let new_d_pn = old_d_pn.replace("\u{2063}", " ");
            *program_name = new_d_pn;
        }
    }

    output_line
}
fn parse(cleandata: String) -> Vec<HashMap<String, String>> {
    let mut raw_output = Vec::new();
    let mut network = false;
    let mut socket = false;
    let mut bluetooth = false;
    let mut routing_table = false;
    let mut interface_table = false;
    let mut headers = None;

    for line in cleandata.lines() {
        if line.starts_with("Active Internet") {
            network = true;
            socket = false;
            bluetooth = false;
            routing_table = false;
            interface_table = false;
            continue;
        }

        if line.starts_with("Active UNIX") {
            network = false;
            socket = true;
            bluetooth = false;
            routing_table = false;
            interface_table = false;
            continue;
        }

        if line.starts_with("Active Bluetooth") {
            network = false;
            socket = false;
            bluetooth = true;
            routing_table = false;
            interface_table = false;
            continue;
        }

        if line.starts_with("Kernel IP routing table") {
            network = false;
            socket = false;
            bluetooth = false;
            routing_table = true;
            interface_table = false;
            continue;
        }

        if line.starts_with("Kernel Interface table") {
            network = false;
            socket = false;
            bluetooth = false;
            routing_table = false;
            interface_table = true;
            continue;
        }

        // get headers
        if line.starts_with("Proto") {
            let header_text = normalize_headers(line.to_owned());
            headers = Some(
                header_text
                    .split_whitespace()
                    .map(|x| x.to_string())
                    .collect(),
            );
            continue;
        }

        if line.starts_with("Destination ") {
            let header_text = normalize_route_headers(line.to_owned());
            headers = Some(
                header_text
                    .split_whitespace()
                    .map(|x| x.to_string())
                    .collect(),
            );
            continue;
        }

        
        if line.starts_with("Iface ") {
            let header_text = normalize_interface_headers(line.to_owned());
            headers = Some(
                header_text
                    .split_whitespace()
                    .map(|x| x.to_string())
                    .collect(),
            );
            continue;
        }

        // parse items
        if network {
            raw_output.push(parse_network(&headers.as_ref().unwrap(), line));
            continue;
        }

        if socket {
            raw_output.push(parse_socket(&header_text, &headers.as_ref().unwrap(), line));
            continue;
        }

        if bluetooth {
            // not implemented
            todo!();
        }

        if routing_table {
            raw_output.push(parse_route(&headers.as_ref().unwrap(), line));
            continue;
        }

        if interface_table {
            raw_output.push(parse_interface(&headers.as_ref().unwrap(), line));
            continue;
        }
    }

    return parse_post(raw_output);
}
