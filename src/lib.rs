use std::collections::HashMap;
use std::default::Default;
use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Behavior {
    TwoWaySwitch,
    ThreeWaySwitch,
    FourWaySwitch,
    FiveWaySwitch,
    SixWaySwitch,
    SevenWaySwitch,
    EgithWaySwitch,
    Slider,
    ReversableSlider,
    HVAC,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

struct DeviceSynonyms {
    device_group: DeviceGroup,
    name: &'static str,
    uuid_number: u128,
}

pub const DEVICE_GROUPS: [DeviceSynonyms; 2] = [
    DeviceSynonyms {
        device_group: DeviceGroup::Light,
        name: "lights",
        uuid_number: 0xf1d34301c91642a88c7c274828177649,
    },
    DeviceSynonyms {
        device_group: DeviceGroup::Fan,
        name: "fans",
        uuid_number: 0x3d39295fb06842ecabeed69e0d65c105,
    }
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum DeviceGroup{
    Light,
    Fan,
}

struct ActionSynonyms {
    action: Action,
    name: &'static str,
    uuid_number: u128,
}

const ACTION_SYNONYMS: [ActionSynonyms; 8] = [
    ActionSynonyms {
        action: Action::On,
        name: "on",
        uuid_number: 0x928e9b929939486b998d69613f89a9a6,
    },
    ActionSynonyms {
        action: Action::Off,
        name: "off",
        uuid_number: 0x13df417d74d2443b87e3de60557b75b8,
    },
    ActionSynonyms {
        action: Action::Up(None),
        name: "up",
        uuid_number: 0xbc6c6eeba0ba40e0a57ff5186d4350ce,
    },
    ActionSynonyms {
        action: Action::Down(None),
        name: "down",
        uuid_number: 0x62865402c86245eea282d4f2ca8fd51b,
    },
    ActionSynonyms {
        action: Action::Min,
        name: "minimum",
        uuid_number: 0x4aad1b26ea9b455190d0d917102b7f36,
    },
    ActionSynonyms {
        action: Action::Max,
        name: "maximum",
        uuid_number: 0x4ffb631fa4ba4fb5a189f7a3bb9dfa01,
    },
    ActionSynonyms {
        action: Action::Reverse,
        name: "reverse",
        uuid_number: 0x1a8a1df0523e4acb8390b872329a9ca7,
    },
    ActionSynonyms {
        action: Action::Set(0),
        name: "set",
        uuid_number: 0x2a4fae8107134e1fa8187ac56e4f13e4,
    },
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Action {
    On,
    Off,
    Up(Option<usize>),
    Down(Option<usize>),
    Min,
    Max,
    Reverse,
    Set(usize),
}

impl Action {
    fn same_variant(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }

    pub fn from_str(s: &str, target: Option<usize>) -> Result<Self, &'static str> {
        let s = s.to_lowercase();

        match s.as_str() {
            "up" => {
                return Ok(Action::Up(target));
            }
            "down" => {
                return Ok(Action::Down(target));
            }
            "set" => {
                if target.is_some() {
                    return Ok(Action::Set(target.unwrap()));
                } else {
                    return Err("No target was given");
                }
            }
            name => {
                for action in ACTION_SYNONYMS {
                    if action.name == name {
                        return Ok(action.action);
                    }
                }
            }
        }
        Err("Bad Action name given")
    }

    pub fn from_u128(uuid_number: u128, target: Option<usize>) -> Result<Self, &'static str> {
        for action_synonym in ACTION_SYNONYMS {
            if action_synonym.uuid_number == uuid_number {
                return Self::from_str(action_synonym.name, target);
            }
        }
        Err("Bad Uuid number given, no associated action")
    }

    pub fn to_str(&self) -> &'static str {
        use Action as A;
        for action_synonym in ACTION_SYNONYMS {
            if self.same_variant(&action_synonym.action) {
                return action_synonym.name;
            }
        }
        ""
    }

    pub fn to_uuid(&self) -> Uuid {
        use Action as A;
        for action_synonym in ACTION_SYNONYMS {
            if self.same_variant(&action_synonym.action) {
                return Uuid::from_u128(action_synonym.uuid_number);
            }
        }
        Uuid::from_u128(0x0)
    }

    pub fn get_value(&self) -> Option<usize> {
        match self {
            Action::Up(v) => v.clone(),
            Action::Down(v) => v.clone(),
            Action::Set(v) => Some(v.clone()),
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
/// could calculate max number of duty_cycles, check that they're in the right order, check that
/// available actions Set has the right default value
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
    pub duty_cycles: [Option<u32>; 8],
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
    pub device_group: Option<DeviceGroup>,
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
                Action::Up(None),
                Action::Down(None),
                Action::Min,
                Action::Max,
                Action::Set(0),
            ]),
            default_target: 3,
            duty_cycles: [Some(0), Some(2), Some(4), Some(8), Some(16), Some(32), Some(64), Some(96)],
            target: 0,
            freq_Hz: 100,
            device_group: None,
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
        duty_cycles: [Option<u32>; 8],
        target: usize,
        freq_Hz: u32,
        device_group: Option<DeviceGroup>,
        reversed: bool,
        updated: bool,
        behavior: Behavior,
    ) -> Self {
        Self {
            uuid,
            name,
            action: Action::Off,
            available_actions: Vec::from([
                Action::On,
                Action::Off,
                Action::Up(None),
                Action::Down(None),
                Action::Min,
                Action::Max,
                Action::Set(0),
            ]),
            default_target,
            duty_cycles,
            target,
            freq_Hz,
            device_group,
            reversed,
            updated: true,
            behavior,
        }
    }

    fn get_max_usable_duty_cycle_index(&self) -> usize {
        self.duty_cycles.iter().filter(|x| x.is_some()).count() - 1
    }

    pub fn from_json(json: &String) -> Result<Self, &'static str> {
        let device: Result<Device, serde_json::Error> = serde_json::from_str(json);
        match device {
            Ok(d) => Ok(d),
            Err(e) => {
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
        use Action as A;
        match action {
            A::On => {
                if !self.available_actions.contains(&action) {
                    panic!("Action not available fror device");
                }
                self.target = self.default_target;
            }
            A::Off => {
                if !self.available_actions.contains(&action) {
                    panic!("Action not available fror device");
                }
                self.target = 0;
            }
            A::Up(v) => {
                if !self.available_actions.contains(&Action::Up(None)) {
                    panic!("Action not available fror device");
                }
                let amount = match v {
                    Some(a) => a,
                    None => 1,
                };
                self.target = (self.target + amount)
                    .min(self.get_max_usable_duty_cycle_index());
            }
            A::Down(v) => {
                if !self.available_actions.contains(&Action::Down(None)) {
                    panic!("Action not available fror device");
                }
                let amount = match v {
                    Some(a) => a,
                    None => 1,
                };
                self.target = if amount < self.target {
                    self.target - amount
                } else {
                    0
                };
            }
            A::Min => {
                if !self.available_actions.contains(&action) {
                    panic!("Action not available fror device");
                }
                self.target = 1;
            }
            A::Max => {
                if !self.available_actions.contains(&action) {
                    panic!("Action not available fror device");
                }
                self.target = self.get_max_usable_duty_cycle_index();
            }
            A::Reverse => {
                if !self.available_actions.contains(&action) {
                    panic!("Action not available fror device");
                }
                self.reversed = !self.reversed;
            }
            A::Set(v) => {
                if !self.available_actions.contains(&Action::Set(0)) {
                    panic!("Action not available fror device");
                }
                self.target = v.min(self.get_max_usable_duty_cycle_index()); 
            }
        }
        self.action = action;
        self.updated = true;
        Ok(())
    }

    pub fn get_duty_cycle(&self) -> u32 {
        self.duty_cycles[self.target].unwrap()
    }
}

pub struct Devices {
    pub devices: Arc<Mutex<Vec<Device>>>,
}

impl Devices {
    fn append(&mut self, other: &mut Self) {
        let mut self_guard = self.devices.lock().unwrap();
        let mut other_guard = other.devices.lock().unwrap();
        self_guard.append(&mut other_guard);
    }

    fn new(devices: Arc<Mutex<Vec<Device>>>) -> Self {
        Self { devices }
    }

    pub fn clone(&self) -> Self {
        Self {
            devices: Arc::clone(&self.devices)
        }
    }
}

/*
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
//     Set,*/
