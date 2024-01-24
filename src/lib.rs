use std::collections::HashMap;

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json;

const ACTIONS: [(Action, &'static str, u128); 8] = [
    (Action::On, "on", 0x928e9b929939486b998d69613f89a9a6),
    (Action::Off, "off", 0x13df417d74d2443b87e3de60557b75b8),
    (Action::Up, "up", 0xbc6c6eeba0ba40e0a57ff5186d4350ce),
    (Action::Down, "down", 0x62865402c86245eea282d4f2ca8fd51b),
    (Action::Min, "minimum", 0x4aad1b26ea9b455190d0d917102b7f36),
    (Action::Max, "maximum", 0x4ffb631fa4ba4fb5a189f7a3bb9dfa01),
    (Action::Reverse, "reverse", 0xa201801c1cbe4c918873c04486d3208b),
    (Action::Set, "set", 0x2a4fae8107134e1fa8187ac56e4f13e4),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Action {
    On,
    Off,
    Up,
    Down,
    Min,
    Max,
    Reverse,
    Set,
}

impl Action {
    pub fn from_str(s: &str) -> Result<Self, &str> {
        let s = s.to_lowercase();
        
        let action_set: HashMap<&str, Action> = ACTIONS.iter().map(|(d, s, _)| (*s, *d)).collect();

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

    pub fn from_u128(&self, num: u128) -> Result<Self, &str> {
        for (a, _, n) in ACTIONS.iter() {
            if *n == num {
                return Ok(a.clone());
            }
        }
        Err("Bad Uuid number given, no associated action")
    }

    pub fn to_uuid(&self) -> Uuid {
        for (a, _, n) in ACTIONS.iter() {
            if a == self {
                return Uuid::from_u128(n)
            }
        }
        Uuid::from_u128(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Device {
    pub uuid: Uuid,
    pub name: String,
    pub action: Action,
    pub available_actions: Vec<Action>,
    pub default_target: usize,
    pub duty_cycles: [u32; 8],
    pub target: usize,
    pub freq_Hz: u32,
    pub reversed: bool,
    pub updated: bool,
}

impl Device {
    pub fn from_json(json: &String) -> Result<Self, &'static str> {
        let device: Result<Device, serde_json::Error> = serde_json::from_str(json);
        match device {
            Ok(d) => Ok(d),
            Err(e) => {
                println!("000000000000 {}", e.to_string().as_str());
                Err("Could not convert Device to json")
            }
        }
    }

    pub fn to_json(&self) -> String {
        let result = serde_json::to_string(&self);

        match result {
            Ok(j) => j,
            Err(_) => String::from("somehting went wrong")
        }
    }

    pub fn take_action(&mut self, action: Action, target: Option<usize>) -> Result<(), &'static str>{
        if !self.available_actions.contains(&self.action) {
            return Err("Action not available for device")
        }
        use Action::*;
        match action {
            On => {
                self.target = self.default_target;
            },
            Off => {
                self.target = 0;
            },
            Up => {
                self.target = (self.target + 1).min(self.duty_cycles.len() - 1);
            }
            Down => {
                self.target = if 1 < self.target {
                    self.target - 1
                } else {
                    0
                };
            },
            Min => {
                self.target = 1;
            },
            Max => {
                self.target = self.duty_cycles.len() - 1;
            },
            Reverse => {
                self.reversed = !self.reversed;
            },
            Set => {
                let target = target.ok_or("invalid target")?; 
                self.target = if target > self.duty_cycles.len() - 1  {
                    self.duty_cycles.len() - 1
                } else {
                    target
                };
            },          
        }
        self.action = action;
        self.updated = true;
        Ok(())
    }

    pub fn get_duty_cycle(&self) -> u32 {
        self.duty_cycles[self.target]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_to_json() {
        use Action::*;
        let device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            freq_kHz: 10,
        };

        let jsoned = device.to_json();

        let actual =  "{\"name\":\"Device1\",\"action\":\"Off\",\"available_actions\":[\"On\",\"Off\",\"Min\",\"Max\"],\"default_target\":3,\"duty_cycles\":[0,10,45,60,80,90],\"target\":2,\"freq_kHz\":10}";

        assert_eq!(jsoned, actual);
    }

    #[test]
    fn device_from_json() {
        use Action::*;
        let device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            freq_kHz: 10,
        };

        let json =  "{\"name\":\"Device1\",\"action\":\"Off\",\"available_actions\":[\"On\",\"Off\",\"Min\",\"Max\"],\"default_target\":3,\"duty_cycles\":[0,10,45,60,80,90],\"target\":2,\"freq_kHz\":10}";

        let actual = Device::from_json(&json.to_string());

        assert_eq!(device, actual.unwrap());
    }

    #[test]
    fn take_action_on() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            freq_kHz: 110,
        };

        device.take_action(On, None);

        assert_eq!(device.target, 3);
        assert_eq!(device.get_duty_cycle(), 60);
        assert_eq!(device.action, On);
    }
    
    #[test]
    fn take_action_off() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::On,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            freq_kHz: 110,
        };

        device.take_action(Off, None);

        assert_eq!(device.target, 0);
        assert_eq!(device.get_duty_cycle(), 0);
        assert_eq!(device.action, Off);
    }

    #[test]
    fn take_action_up() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 5,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            freq_kHz: 110,
        };

        device.take_action(Up, None);

        assert_eq!(device.target, 3);
        assert_eq!(device.get_duty_cycle(), 60);
        assert_eq!(device.action, Up);
    }

    #[test]
    fn take_action_up_already_max() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 5,
            freq_kHz: 110,
        };

        device.take_action(Up, None);

        assert_eq!(device.target, 5);
        assert_eq!(device.get_duty_cycle(), 90);
        assert_eq!(device.action, Up);
    }

    #[test]
    fn take_action_down() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 5,
            freq_kHz: 110,
        };

        device.take_action(Down, None);

        assert_eq!(device.target, 4);
        assert_eq!(device.get_duty_cycle(), 80);
        assert_eq!(device.action, Down);
    }

    #[test]
    fn take_action_down_already_down() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::On,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 0,
            freq_kHz: 110,
        };

        device.take_action(Down, None);

        assert_eq!(device.target, 0);
        assert_eq!(device.get_duty_cycle(), 0);
        assert_eq!(device.action, Down);
    }

    #[test]
    fn take_action_min() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            freq_kHz: 110,
        };

        device.take_action(Min, None);

        assert_eq!(device.target, 1);
        assert_eq!(device.get_duty_cycle(), 10);
        assert_eq!(device.action, Min);
    }

    #[test]
    fn take_action_max() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            freq_kHz: 110,
        };

        device.take_action(Max, None);

        assert_eq!(device.target, 5);
        assert_eq!(device.get_duty_cycle(), 90);
        assert_eq!(device.action, Max);
    }

    #[test]
    fn take_action_set() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            freq_kHz: 110,
        };

        device.take_action(Set, Some(5));

        assert_eq!(device.target, 5);
        assert_eq!(device.get_duty_cycle(), 90);
        assert_eq!(device.action, Set);
    }

    #[test]
    fn take_action_set_high() {
        use Action::*;
        let mut device = Device {
            uuid: Uuid,
            name: String::from("Device1"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            duty_cycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            freq_kHz: 110,
        };

        device.take_action(Set, Some(6));

        assert_eq!(device.target, 5);
        assert_eq!(device.get_duty_cycle(), 90);
        assert_eq!(device.action, Set);
    }
}

// On,
//     Off,
//     Up,
//     Down,
//     Min,
//     Max,
//     Set,
