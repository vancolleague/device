use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const ACTIONS: [(Action, &'static str); 7] = [
    (Action::On, "on"),
    (Action::Off, "off"),
    (Action::Up, "up"),
    (Action::Down, "down"),
    (Action::Min, "min"),
    (Action::Max, "max"),
    (Action::Set, "set"),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    On,
    Off,
    Up,
    Down,
    Min,
    Max,
    Set,
}

impl Action {
    pub fn from_str(s: &str) -> Result<Self, &str> {
        let action_set: HashMap<&str, Action> = ACTIONS.iter().map(|(d, s)| (*s, *d)).collect();

        for (key, &value) in action_set.iter() {
            if s.starts_with(key) {
                return Ok(value)
            }
        }

        Err("Bad Action name given")
    }

    pub fn to_str(&self) -> &str {
        for a in ACTIONS {
            if a.0 == *self {
                return a.1
            }
        }
        ""
    }
}

#[derive(Serialize, Deserialize)]
pub struct Device_Status {
    name: String,
    ip: String,
    action: Action,
    #[serde(default)]
    target: Option<i8>,
    #[serde(default)]
    terget_min: Option<i8>,
    #[serde(default)]
    target_max: Option<i8>,
    #[serde(default)]
    period: Option<f32>,
}

impl Device_Status {
    fn from_json(json: String) -> Result<Self, &'static str> {
        let result: Result<Device_Status, serde_json::Error> = serde_json::from_str(json.as_str());

        match result {
            Ok(d) => Ok(d),
            Err(_) => Err("Device couldn't be initialized from Json data"),
        }
    }

    fn to_json(&self) -> String {
        let result = serde_json::to_string(&self);

        match result {
            Ok(j) => j,
            Err(_) => String::from("somehting went wrong")
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
