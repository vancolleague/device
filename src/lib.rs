use std::collections::HashMap;
use std::default::Default;

use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Behavior {
    OnOff,
    ReversableOnOff,
    Slider,
    ReversableSlider,
}

pub const DEVICE_TYPES: [(DeviceType, &'static str, u128); 3] = [
    (
        DeviceType::Light,
        "lights",
        0xf1d34301c91642a88c7c274828177649,
    ),
    (DeviceType::Fan, "fans", 0x3d39295fb06842ecabeed69e0d65c105),
    (
        DeviceType::Generic,
        "generic",
        0x36715f57d8c6400d91f403cc1f20c793,
    ),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum DeviceType {
    Light,
    Fan,
    Generic,
}

const ACTIONS: [(Action, &'static str, u128); 8] = [
    (Action::On, "on", 0x928e9b929939486b998d69613f89a9a6),
    (Action::Off, "off", 0x13df417d74d2443b87e3de60557b75b8),
    (
        Action::Up { amount: None },
        "up",
        0xbc6c6eeba0ba40e0a57ff5186d4350ce,
    ),
    (
        Action::Down { amount: None },
        "down",
        0x62865402c86245eea282d4f2ca8fd51b,
    ),
    (Action::Min, "minimum", 0x4aad1b26ea9b455190d0d917102b7f36),
    (Action::Max, "maximum", 0x4ffb631fa4ba4fb5a189f7a3bb9dfa01),
    (
        Action::Reverse,
        "reverse",
        0xa201801c1cbe4c918873c04486d3208b,
    ),
    (
        Action::Set { target: 0 },
        "set",
        0x2a4fae8107134e1fa8187ac56e4f13e4,
    ),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Action {
    On,
    Off,
    Up { amount: Option<usize> },
    Down { amount: Option<usize> },
    Min,
    Max,
    Reverse,
    Set { target: usize },
}

impl Action {
    pub fn from_str(s: &str, target: Option<usize>) -> Result<Self, &'static str> {
        let s = s.to_lowercase();

        if s == "up" {
            return Ok(Action::Up { amount: target });
        }

        if s == "down" {
            return Ok(Action::Down { amount: target });
        }

        if s == "set" && target.is_some() {
            return Ok(Action::Set {
                target: target.unwrap(),
            });
        }

        let action_set: HashMap<&str, Action> = ACTIONS.iter().map(|(a, t, _)| (*t, *a)).collect();

        for (key, &value) in action_set.iter() {
            if s.starts_with(key) {
                return Ok(value);
            }
        }

        Err("Bad Action name given")
    }

    pub fn to_str(&self) -> &str {
        match *self {
            Action::Up{ .. } => "up",
            Action::Down { .. } => "down",
            Action::Set { .. } => "set",
            _ => {
                for a in ACTIONS {
                    if a.0 == *self {
                        return a.1;
                    }
                }
                ""
            }
        }
    }

    pub fn from_u128(num: u128, target: Option<usize>) -> Result<Self, &'static str> {
        if ACTIONS[7].2 == num && target.is_some() {
            let target = target.unwrap();
            if target > 7 {
                return Err("Target is too large");
            } else {
                return Ok(Action::Set { target: target });
            }
        }

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
                return Uuid::from_u128(n.clone());
            }
        }
        Uuid::from_u128(1)
    }

    pub fn get_target(&self) -> Option<usize> {
        match self {
            Action::Set { target: a } => Some(a.clone()),
            _ => None,
        }
    }

    fn get_amount(&self) -> Option<usize> {
        match self {
            Action::Up { amount: a } => a.clone(),
            Action::Down { amount: a } => a.clone(),
            _ => None,
        }
    }

    pub fn get_target_or_amount(&self) -> Option<usize> {
        match self {
            Action::Up { amount: a } => a.clone(),
            Action::Down { amount: a } => a.clone(),
            Action::Set { target: a } => Some(a.clone()),
            _ => None,
        }
    }
}
/// Represents a device on a node
///
/// While custom behaviors can be generated, its assumed to control a PWM based device. The
/// available duty cycles are stored in 'duty_cycles' which is the percent of the time that device
/// will be on. 'target' specifiies which duty cycle is currently selected. Actions control the
/// selection of targets/duty cycles.
///
/// # Examples
///
/// ```
/// let device = Device::new("fan", Uuid::from_u128(0xf1d34301c91642a88c7c274828177649));
/// println!("Device: {:?}", device);
/// `
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Device {
    /// Unique identifier for the device.
    ///
    /// This field must be supplied when creating a new 'Device' and does not have a default value
    pub uuid: Uuid,
    /// A human-readable name for the device.
    ///
    /// This field must be supplied when creating a new 'Device' and does not have a default value.
    /// It must also have a unique value on the network.
    pub name: String,
    /// What the device is about to do or last did. Optional
    ///
    /// Defaults to 'Off'. Can be set using 'with_action'.
    pub action: Action,
    /// What the device can do, valid values for 'Action'. Optional
    ///
    /// Defaults to Vec::from([On, Off, Up, Down, Min, Max, Set { target: 0 },]).
    /// Can be set using 'with_available_actions'
    pub available_actions: Vec<Action>,
    /// The default 'target', to be used in conjunction with the 'On' 'Action'.
    ///
    /// Defaults to 3. Must be <= 7. Can be set using 'with_default_target'.
    pub default_target: usize,
    /// The array of duty cycles that are targetable by the device.
    ///
    /// Devaults to [0, 2, 4, 8, 16, 32, 64, 96]. 100 can cause problems for some hardware.
    /// Must be exactly 8 cells long and each cell must be in the inclusive range of 0 though 100. Can be set using 'with_duty_cycles'
    pub duty_cycles: [u32; 8],
    /// The index of the duty cycle from the 'duty_cycles' array that's currently to be targetted.
    ///
    /// Defaults to 3. Must by in the inclusive 0 to 7 range. Can be set using 'with_target'.
    pub target: usize,
    /// The frequency that the PWM will operate at in Hz.
    ///
    /// Defaults to 1000. Can be set using 'with_freq_Hz'.
    pub freq_Hz: u32,
    /// The type of device, used for addressing groups of devices such as lights or fans.
    ///
    /// Defaults to Generic which is meant to be used for devices that aren't to be grouped. Can be
    /// set using 'with_device_type'.
    pub device_type: DeviceType,
    /// Used for controlling the directon of reversable devices.
    ///
    /// Could be used for the direction of a fan or Heat vs. Cool in an HVAC system.
    ///
    /// Defaults to 'false'. Can be set using 'with_reversed'.
    pub reversed: bool,
    /// Used for tracking when updates have been made to a devices state for the sake of ttrigering
    /// other changes.
    ///
    /// Can be used to signify when things such as PWM duty cycles must be updated.
    ///
    /// Defaults to 'true', this can be used to set initial configurations of underlying hardware.
    /// Can be set using 'with_updated'.
    pub updated: bool,
    pub behavior: Behavior,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            uuid: Uuid::from_u128(0x0),
            name: "".to_string(),
            action: Action::Off,
            available_actions: Vec::from([
                Action::On,
                Action::Off,
                Action::Up { amount: None },
                Action::Down { amount: None },
                Action::Min,
                Action::Max,
                Action::Set { target: 0 },
            ]),
            default_target: 3,
            duty_cycles: [0, 2, 4, 8, 16, 32, 64, 96],
            target: 0,
            freq_Hz: 1000,
            device_type: DeviceType::Generic,
            reversed: false,
            updated: true,
            behavior: Behavior::Slider,
        }
    }
}

impl Device {
    /// Constructs a new 'Device' with the given 'uuid' and 'name'.
    /// All other properties are optional and will be filled with defaults unless relevent
    /// functions are used.
    fn new(
        uuid: Uuid,
        name: String,
        action: Action,
        available_actions: Vec<Action>,
        default_target: usize,
        duty_cycles: [u32; 8],
        target: usize,
        freq_kHz: u32,
        device_type: DeviceType,
        reversed: bool,
        updated: bool,
    ) -> Self {
        Self {
            uuid,
            name,
            action: Action::Off,
            available_actions: Vec::from([
                Action::On,
                Action::Off,
                Action::Up { amount: None },
                Action::Down { amount: None },
                Action::Min,
                Action::Max,
                Action::Set { target: 0 },
            ]),
            default_target: 3,
            duty_cycles: [0, 2, 4, 8, 16, 32, 64, 96],
            target: 0,
            freq_Hz: 1000,
            device_type: DeviceType::Generic,
            reversed: false,
            updated: true,
            behavior: Behavior::Slider,
        }
    }

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
            Err(_) => String::from("somehting went wrong"),
        }
    }

    pub fn take_action(&mut self, action: Action) -> Result<(), &'static str> {
        if let Action::Up { .. } = action {
            let mut up_present = false;
            for a in self.available_actions.iter() {
                if let Action::Up { .. } = a {
                    up_present = true;
                }
            }
            if !up_present {
                return Err("Action not available for device");
            }
        } else if let Action::Down { .. } = action {
            let mut down_present = false;
            for a in self.available_actions.iter() {
                if let Action::Down { .. } = a {
                    down_present = true;
                }
            }
            if !down_present {
                return Err("Action not available for device");
            }
        } else if let Action::Set { .. } = action {
            let mut set_present = false;
            for a in self.available_actions.iter() {
                if let Action::Set { .. } = a {
                    set_present = true;
                }
            }
            if !set_present {
                return Err("Action not available for device");
            }
        } else if !self.available_actions.contains(&action) {
            return Err("Action not available for device");
        }
        use Action::*;
        match action {
            On => {
                self.target = self.default_target;
            }
            Off => {
                self.target = 0;
            }
            Up { .. } => {
                let amount = action.get_amount();
                let amount = match amount {
                    Some(a) => a,
                    None => 1,
                };
                self.target = (self.target + amount).min(self.duty_cycles.len() - 1);
            }
            Down { .. } => {
                let amount = action.get_amount();
                let amount = match amount {
                    Some(a) => a,
                    None => 1,
                };
                // can't use the "Up" process because it'll underflow sometimes
                self.target = if amount < self.target {
                    self.target - amount
                } else {
                    0
                };
            }
            Min => {
                self.target = 1;
            }
            Max => {
                self.target = self.duty_cycles.len() - 1;
            }
            Reverse => {
                self.reversed = !self.reversed;
            }
            Set { .. } => {
                let target = action.get_target().unwrap();
                self.target = if target > self.duty_cycles.len() - 1 {
                    self.duty_cycles.len() - 1
                } else {
                    target
                };
            }
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
