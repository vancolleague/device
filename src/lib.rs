use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;

const ACTIONS: [(Action, &'static str); 7] = [
    (Action::On, "on"),
    (Action::Off, "off"),
    (Action::Up, "up"),
    (Action::Down, "down"),
    (Action::Min, "min"),
    (Action::Max, "max"),
    (Action::Set, "set"),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Device {
    pub name: String,
    pub action: Action,
    pub available_actions: Vec<Action>,
    pub default_target: usize,
    pub duty_cycles: [u32; 8],
    pub target: usize,
    pub freq_Hz: u32,
}

impl Device {
    pub fn from_json(json: &String) -> Result<Self, &'static str> {
        let device: Result<Device, serde_json::Error> = serde_json::from_str(json);
        match device {
            Ok(d) => Ok(d),
//            Err(e) => Err(format!("{}: couldn't deserialize device json", e.clone()).as_str())
            Err(e) => Err(e.to_string()),
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
