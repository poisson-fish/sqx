// This function will compare to serde_json::Values and decide if they are subsets.
pub fn isSubsetOf(obj_a: &serde_json::Value, obj_b: &serde_json::Value) -> bool {
    match obj_a {
        serde_json::Value::Null => {
            if obj_b.is_null() {
                trace!("Both values are null");
                return true;
            }
            return false;
        }
        serde_json::Value::Bool(_) => {
            if obj_b.is_boolean() {
                trace!("Both values are boolean");
                return true;
            }
            return false;
        }
        serde_json::Value::Number(_) => {
            if obj_b.is_number() {
                trace!("Both values are numbers");
                return true;
            }
            return false;
        }
        serde_json::Value::String(_) => {
            if obj_b.is_string() {
                trace!("Both values are strings");
                return true;
            }
            return false;
        }
        serde_json::Value::Array(_) => {
            let v = obj_a.as_array().unwrap();
            if let Some(obj_b_obj) = obj_b.as_array() {
                for (idx, elem) in v.iter().enumerate() {
                    if let Some(value_b) = obj_b_obj.get(idx) {
                        if !isSubsetOf(elem, value_b) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            else{
                return false;
            }
            return true;
        }
        serde_json::Value::Object(_) => {
            let v = obj_a.as_object().unwrap();
            if let Some(obj_b_obj) = obj_b.as_object() {
                for elem in v {
                    if let Some(value_b) = obj_b_obj.get(elem.0) {
                        if !isSubsetOf(elem.1, value_b) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }
    }
    return true;
}

pub fn includesIn<'a>(obj_list: &Vec<&'a serde_json::Value>, target: &serde_json::Value) -> Option<&'a serde_json::Value> {
    if obj_list.len() <= 0 { return None; }
    for object in obj_list {
        if isSubsetOf(*object, target) {
            return Some(*object);
        }
    }
    return None;
}

pub fn jsonto_statement<'a>(obj: &'a serde_json::Value) -> () {
    let recurse = |obj: &'a serde_json::Value| {
        fn helper<'a>(
            obj: &'a serde_json::Value,
            obj_stack: &mut Vec<String>,
            objects: &mut Vec<&'a serde_json::Value>,
        ) -> () {
            let default_name = String::from("noname");
            match obj {
                serde_json::Value::Null => {
                    let name = obj_stack.pop().unwrap_or(default_name);
                    println!("Got a null value named {}", name);
                }
                serde_json::Value::Bool(_) => {
                    let v = obj.as_bool().unwrap();
                    let name = obj_stack.pop().unwrap_or(default_name);
                    println!("Got a bool: {} named {}", v, name);
                }
                serde_json::Value::Number(_) => {
                    let v = obj.as_f64().unwrap();
                    let name = obj_stack.pop().unwrap_or(default_name);
                    println!("Got a number: {:#?} named {}", v, name);
                }
                serde_json::Value::String(_) => {
                    let v = obj.as_str().unwrap();
                    let name = obj_stack.pop().unwrap_or(default_name);
                    println!("Got a string: {} named {}", v, name);
                }
                serde_json::Value::Array(_) => {
                    let v = obj.as_array().unwrap();
                    let name = obj_stack.pop().unwrap_or(default_name);
                    println!("Got an array with {} elements named {}", v.len(), name);
                    for elem in v {
                        helper(elem, obj_stack, objects);
                    }
                }
                serde_json::Value::Object(_) => {
                    let v = obj.as_object().unwrap();

                    if let Some(found) = includesIn(objects, obj) {
                        println!("I have seen this object before! {}",objects.len());
                    } else {
                        println!("I have NOT seen this object before!");
                        objects.push(obj);
                    }
                    let name = obj_stack.pop().unwrap_or(default_name);
                    println!("Got an object with {} elements named {}", v.len(), name);
                    for elem in v {
                        obj_stack.push(elem.0.clone());
                        helper(elem.1, obj_stack, objects);
                    }
                }
            }
        }

        let mut obj_stack: Vec<String> = Vec::new();
        //       obj_count: 0,
        let mut objects: Vec<&serde_json::Value> = Vec::new();
        helper(obj, &mut obj_stack, &mut objects);
    };
    recurse(obj);
}


#[cfg(test)]
mod tests {

    use crate::converters::json::{isSubsetOf, includesIn};

    #[test]
    fn is_subset_of_eq() {
        let obj_a = serde_json::json!({
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
          let obj_b = serde_json::json!({
            "user": "test",
            "pid": 33,
            "vsz": 49716,
            "rss": 16048,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/usr/lib/systemd/systemd-journald",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
        assert_eq!(isSubsetOf(&obj_a,&obj_b), true);
    }
    #[test]
    fn is_subset_of_true() {
        let obj_a = serde_json::json!({
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
          let obj_b = serde_json::json!({
            "user": "test",
            "pid": 33,
            "vsz": 49716,
            "rss": 16048,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/usr/lib/systemd/systemd-journald",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
        assert_eq!(isSubsetOf(&obj_a,&obj_b), true); // objects can have new fields but cannot miss them
    }
    #[test]
    fn is_subset_of_false() {
        let obj_a = serde_json::json!({
            "user": "test",
            "pid": 33,
            "vsz": 49716,
            "rss": 16048,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/usr/lib/systemd/systemd-journald",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
        let obj_b = serde_json::json!({
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
        assert_eq!(isSubsetOf(&obj_a,&obj_b), false);
    }
    #[test]
    fn is_subset_of_eq_array() {
        let obj_a = serde_json::json!([{
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          },{
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          },
          {
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          }]);
          let obj_b = serde_json::json!([{
            "user": "root",
            "pid": 1,
            "vsz": 168446,
            "rss": 13568,
            "tty": null,
            "stat": "Ss",
            "start": "Jan24",
            "time": "0:00",
            "command": "none",
            "cpu_percent": 0.5,
            "mem_percent": 0.0
          },{
            "user": "test",
            "pid": 1,
            "vsz": 169844,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init8",
            "cpu_percent": 0.6,
            "mem_percent": 0.8666
          },
          {
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          }]);
        assert_eq!(isSubsetOf(&obj_a,&obj_b), true);
    }
    #[test]
    fn array_includes_similar_object() {
        let mut vec_a: Vec<&serde_json::Value> = Vec::new();
        let json0 = serde_json::json!({
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
        vec_a.push(&json0);
        let json1 = serde_json::json!({
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0
          });
          vec_a.push(&json1);
          let json2 = serde_json::json!(
            {
              "user": "root",
              "pid": 1,
              "vsz": 168244,
              "rss": 12648,
              "tty": null,
              "stat": "Ss",
              "start": "Jan22",
              "time": "0:00",
              "command": "/sbin/init",
              "cpu_percent": 0.0,
              "mem_percent": 0.0
            });
          vec_a.push(&json2);
          let obj_b = serde_json::json!({
            "user": "root",
            "pid": 1,
            "vsz": 1629346,
            "rss": 135228,
            "tty": null,
            "stat": "Ss",
            "start": "Jan26",
            "time": "0:00",
            "command": "none",
            "cpu_percent": 0.5,
            "mem_percent": 0.0
          });
        assert_eq!(includesIn(&vec_a,&obj_b).is_some(), true);
    }
    #[test]
    fn array_includes_dissimilar_object() {
        let mut vec_a: Vec<&serde_json::Value> = Vec::new();
        let json0 = serde_json::json!({
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0,
            "duty": 2.0
          });
        vec_a.push(&json0);
        let json1 = serde_json::json!({
            "user": "root",
            "pid": 1,
            "vsz": 168244,
            "rss": 12648,
            "tty": null,
            "stat": "Ss",
            "start": "Jan22",
            "time": "0:00",
            "command": "/sbin/init",
            "cpu_percent": 0.0,
            "mem_percent": 0.0,
            "duty": 1.2
          });
          vec_a.push(&json1);
          let json2 = serde_json::json!(
            {
              "user": "root",
              "pid": 1,
              "vsz": 168244,
              "rss": 12648,
              "tty": null,
              "stat": "Ss",
              "start": "Jan22",
              "time": "0:00",
              "command": "/sbin/init",
              "cpu_percent": 0.0,
              "mem_percent": 0.0,
              "duty": 1.0
            });
          vec_a.push(&json2);
          let obj_b = serde_json::json!({
            "user": "root",
          });
        assert_eq!(includesIn(&vec_a,&obj_b).is_some(), false);
    }
}
