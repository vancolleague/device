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
    pub ip: String,
    pub action: Action,
    pub available_actions: Vec<Action>,
    pub default_target: usize,
    pub dutycycles: [usize; 6],
    pub target: usize,
    pub period_ms: usize,
    pub on_duration_ms: usize,
}

impl Device {
    pub fn from_json(json: &String) -> Result<Self, &'static str> {
        let device: Result<Device, serde_json::Error> = serde_json::from_str(json);
        match device {
            Ok(d) => Ok(d),
            Err(_) => Err("Couldn't deserialize device json")
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
                self.update_on_duration_ms();
            },
            Off => {
                self.target = 0;
                self.update_on_duration_ms();
            },
            Up => {
                self.target = (self.target + 1).min(self.dutycycles.len() - 1);
                self.update_on_duration_ms();
            }
            Down => {
                self.target = if 1 < self.target {
                    self.target - 1
                } else {
                    0
                };
                self.update_on_duration_ms();
            },
            Min => {
                self.target = 1;
                self.update_on_duration_ms();
            },
            Max => {
                self.target = self.dutycycles.len() - 1;
                self.update_on_duration_ms();
            },
            Set => {
                let target = target.ok_or("invalid target")?; 
                self.target = if target > self.dutycycles.len() - 1  {
                    self.dutycycles.len() - 1
                } else {
                    target
                };
                self.update_on_duration_ms();
            },          
        }
        self.action = action;
        Ok(())
    }

    fn update_on_duration_ms(&mut self) {
        let duration = self.period_ms * self.dutycycles[self.target] / 100;
        let percent = duration as f32 / self.period_ms as f32;
        self.on_duration_ms = if percent < 0.025 {
            0
        } else if 0.975 < percent {
            self.period_ms
        } else {
            duration
        }
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
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            period_ms: 100,
            on_duration_ms: 60,
        };

        let jsoned = device.to_json();

        let actual =  "{\"name\":\"Device1\",\"ip\":\"192.168.1.70\",\"action\":\"Off\",\"available_actions\":[\"On\",\"Off\",\"Min\",\"Max\"],\"default_target\":3,\"dutycycles\":[0,10,45,60,80,90],\"target\":2,\"period_ms\":100,\"on_duration_ms\":60}";

        assert_eq!(jsoned, actual);
    }

    #[test]
    fn device_from_json() {
        use Action::*;
        let device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            period_ms: 100,
            on_duration_ms: 60,
        };

        let json =  "{\"name\":\"Device1\",\"ip\":\"192.168.1.70\",\"action\":\"Off\",\"available_actions\":[\"On\",\"Off\",\"Min\",\"Max\"],\"default_target\":3,\"dutycycles\":[0,10,45,60,80,90],\"target\":2,\"period_ms\":100,\"on_duration_ms\":60}";

        let actual = Device::from_json(&json.to_string());

        assert_eq!(device, actual.unwrap());
    }

    #[test]
    fn take_action_on() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(On, None);

        assert_eq!(device.target, 3);
        assert_eq!(device.on_duration_ms, 66);
        assert_eq!(device.action, On);
    }
    
    #[test]
    fn take_action_off() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Off, None);

        assert_eq!(device.target, 0);
        assert_eq!(device.on_duration_ms, 0);
        assert_eq!(device.action, Off);
    }

    #[test]
    fn take_action_up() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 5,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 2,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Up, None);

        assert_eq!(device.target, 3);
        assert_eq!(device.on_duration_ms, 66);
        assert_eq!(device.action, Up);
    }

    #[test]
    fn take_action_up_already_max() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 3,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 5,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Up, None);

        assert_eq!(device.target, 5);
        assert_eq!(device.on_duration_ms, 99);
        assert_eq!(device.action, Up);
    }

    #[test]
    fn take_action_down() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 5,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Down, None);

        assert_eq!(device.target, 4);
        assert_eq!(device.on_duration_ms, 88);
        assert_eq!(device.action, Down);
    }

    #[test]
    fn take_action_down_already_down() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 0,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Down, None);

        assert_eq!(device.target, 0);
        assert_eq!(device.on_duration_ms, 0);
        assert_eq!(device.action, Down);
    }

    #[test]
    fn take_action_min() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Min, None);

        assert_eq!(device.target, 1);
        assert_eq!(device.on_duration_ms, 11);
        assert_eq!(device.action, Min);
    }

    #[test]
    fn take_action_max() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Max, None);

        assert_eq!(device.target, 5);
        assert_eq!(device.on_duration_ms, 99);
        assert_eq!(device.action, Max);
    }

    #[test]
    fn take_action_set() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Set, Some(5));

        assert_eq!(device.target, 5);
        assert_eq!(device.on_duration_ms, 99);
        assert_eq!(device.action, Set);
    }

    #[test]
    fn take_action_set_high() {
        use Action::*;
        let mut device = Device {
            name: String::from("Device1"),
            ip: String::from("192.168.1.70"),
            action: Action::Off,
            available_actions: Vec::from([On, Off, Min, Max]),
            default_target: 2,
            dutycycles: [0, 10, 45, 60, 80, 90],
            target: 3,
            period_ms: 110,
            on_duration_ms: 60,
        };

        device.take_action(Set, Some(6));

        assert_eq!(device.target, 5);
        assert_eq!(device.on_duration_ms, 99);
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
