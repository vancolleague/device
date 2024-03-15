#![feature(variant_count)]

use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug)]
pub struct DeviceSynonyms {
    pub device_group: DeviceGroup,
    pub name: &'static str,
    pub uuid_number: u128,
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
    },
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum DeviceGroup {
    Light,
    Fan,
}

#[derive(Debug)]
struct ActionSynonyms {
    action: Action,
    text: &'static str,
    uuid_number: u128,
}

const ACTION_SYNONYMS: [ActionSynonyms; 8] = [
    ActionSynonyms {
        action: Action::On,
        text: "on",
        uuid_number: 0x928e9b929939486b998d69613f89a9a6,
    },
    ActionSynonyms {
        action: Action::Off,
        text: "off",
        uuid_number: 0x13df417d74d2443b87e3de60557b75b8,
    },
    ActionSynonyms {
        action: Action::Up(None),
        text: "up",
        uuid_number: 0xbc6c6eeba0ba40e0a57ff5186d4350ce,
    },
    ActionSynonyms {
        action: Action::Down(None),
        text: "down",
        uuid_number: 0x62865402c86245eea282d4f2ca8fd51b,
    },
    ActionSynonyms {
        action: Action::Min,
        text: "minimum",
        uuid_number: 0x4aad1b26ea9b455190d0d917102b7f36,
    },
    ActionSynonyms {
        action: Action::Max,
        text: "maximum",
        uuid_number: 0x4ffb631fa4ba4fb5a189f7a3bb9dfa01,
    },
    ActionSynonyms {
        action: Action::Reverse,
        text: "reverse",
        uuid_number: 0x1a8a1df0523e4acb8390b872329a9ca7,
    },
    ActionSynonyms {
        action: Action::Set(0),
        text: "set",
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
            text => {
                for synonym in ACTION_SYNONYMS {
                    if synonym.text == text {
                        return Ok(synonym.action);
                    }
                }
            }
        }
        Err("Bad Action text given")
    }

    pub fn from_u128(uuid_number: u128, target: Option<usize>) -> Result<Self, &'static str> {
        for action_synonym in ACTION_SYNONYMS {
            if action_synonym.uuid_number == uuid_number {
                return Self::from_str(action_synonym.text, target);
            }
        }
        Err("Bad Uuid number given, no associated action")
    }

    pub fn to_str(&self) -> &'static str {
        for action_synonym in ACTION_SYNONYMS {
            if self.same_variant(&action_synonym.action) {
                return action_synonym.text;
            }
        }
        ""
    }

    pub fn to_uuid(&self) -> Uuid {
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
/// While using setter functions, error checking and validation is done. As a result, since
/// checks are done in the order that setter functions are called, some valed sets of
/// properties will fail validation if they're assigned in the wrong order.
///
/// # Examples
///
/// ```
/// use device::Device;
/// use uuid::Uuid;
///
/// let device = Device::build(Uuid::from_u128(0xf1d34301c91642a88c7c274828177649), "fan".to_string());
/// println!("Device: {:?}", device);
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
    available_actions: Vec<Action>,
    /// The default 'target', to be used in conjunction with the 'On' 'Action'.
    ///
    /// Defaults to 3. Must be <= 7. Can be set using 'with_default_target'.
    default_target: usize,
    /// The array of duty cycles that are targetable by the device.
    ///
    /// Devaults to [0, 2, 4, 8, 16, 32, 64, 96]. 100 can cause problems for some hardware.
    /// Must be exactly 8 cells long and each cell must be in the inclusive range of 0 though 100. Can be set using 'with_duty_cycles'
    duty_cycles: [Option<u32>; 8],
    max_duty_cycle_index: usize,
    /// The index of the duty cycle from the 'duty_cycles' array that's currently to be targetted.
    ///
    /// Defaults to 3. Must by in the inclusive 0 to 7 range. Can be set using 'with_target'.
    target: usize,
    /// The frequency that the PWM will operate at in Hz.
    ///
    /// Defaults to 1000. Can be set using 'with_freq_Hz'.
    #[allow(non_snake_case)]
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
    updated: bool,
}

impl Device {
    /// Constructs a new 'Device' with the given 'uuid' and 'name'.
    /// All other properties are optional and will be filled with defaults unless relevent
    /// functions are used.
    pub fn build(uuid: Uuid, name: String) -> Result<Self, &'static str> {
        let duty_cycles = [
            Some(0),
            Some(2),
            Some(4),
            Some(8),
            Some(16),
            Some(32),
            Some(64),
            Some(96),
        ];
        let max_duty_cycle_index = Self::get_max_duty_cycle_index(&duty_cycles)?;

        Ok(Self {
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
            default_target: 3,
            duty_cycles,
            max_duty_cycle_index,
            target: 0,
            freq_Hz: 100,
            device_group: None,
            reversed: false,
            updated: true,
        })
    }

    pub fn action(mut self, action: Action) -> Result<Self, &'static str> {
        self.action = action;
        Ok(self)
    }

    pub fn available_actions(
        mut self,
        available_actions: Vec<Action>,
    ) -> Result<Self, &'static str> {
        use Action as A;
        for action in available_actions.iter() {
            match action {
                A::Up(Some(_)) => {
                    return Err("If Action::Up is an an available_action, it must be set to Action::Up(None)");
                }
                A::Down(Some(_)) => {
                    return Err("If Action::Down is an an available_action, it must be set to Action::Down(None)");
                }
                A::Set(v) => {
                    if v != &0 {
                        return Err("If Action::Set is an an available_action, it must be set to Action::Set(0)");
                    }
                }
                _ => {}
            }
        }
        self.available_actions = available_actions;
        Ok(self)
    }

    pub fn get_available_actions(&self) -> &Vec<Action> {
        &self.available_actions
    }

    pub fn default_target(mut self, default_target: usize) -> Result<Self, &'static str> {
        if default_target > self.max_duty_cycle_index {
            return Err(
                "The default_target must not be greater than max_duty_cycle_index,
                   duty_cycles must have a Some value at the default_value index.",
            );
        }
        self.default_target = default_target;
        Ok(self)
    }

    pub fn get_default_target(&self) -> usize {
        self.default_target
    }

    pub fn duty_cycles(mut self, duty_cycles: [Option<u32>; 8]) -> Result<Self, &'static str> {
        let max_duty_cycle_index = Device::get_max_duty_cycle_index(&duty_cycles)?;
        if self.default_target > max_duty_cycle_index || self.target > max_duty_cycle_index {
            return Err(
                "The default_target and target must not be greater than max_duty_cycle_index,
                   duty_cycles must have a Some value at the default_value index.",
            );
        }
        self.duty_cycles = duty_cycles;
        self.max_duty_cycle_index = max_duty_cycle_index;
        Ok(self)
    }

    // TODO: test
    pub fn get_duty_cycles(&self) -> &[Option<u32>; 8] {
        &self.duty_cycles
    }

    pub fn target(mut self, target: usize) -> Result<Self, &'static str> {
        if target > self.max_duty_cycle_index {
            return Err("The target must not be greater than max_duty_cycle_index,
duty_cycles must have a Some value at the default_value index.");
        }
        self.target = target;
        Ok(self)
    }

    pub fn get_target(&self) -> usize {
        self.target
    }

    pub fn freq_Hz(mut self, freq: u32) -> Result<Self, &'static str> {
        self.freq_Hz = freq;
        Ok(self)
    }

    pub fn device_group(mut self, device_group: Option<DeviceGroup>) -> Result<Self, &'static str> {
        self.device_group = device_group;
        Ok(self)
    }

    fn get_max_duty_cycle_index(duty_cycles: &[Option<u32>; 8]) -> Result<usize, &'static str> {
        let mut some_count = 0;
        let mut found_none = false;
        for dc in duty_cycles {
            if dc.is_some() {
                some_count += 1;
                if found_none {
                    return Err("Within the array of duty_cycles, there mustn't be a Some value that follows a None.");
                }
            } else {
                found_none = true;
            }
        }
        Ok(some_count - 1)
    }

    pub fn from_json(json: &String) -> Result<Self, &'static str> {
        let device: Result<Device, serde_json::Error> = serde_json::from_str(json);
        match device {
            Ok(d) => Ok(d),
            Err(_) => Err("Could not convert Device to json"),
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
                    return Err("Action not available for device.");
                }
                self.target = self.default_target;
            }
            A::Off => {
                if !self.available_actions.contains(&action) {
                    return Err("Action not available for device.");
                }
                self.target = 0;
            }
            A::Up(v) => {
                if !self.available_actions.contains(&Action::Up(None)) {
                    return Err("Action not available for device.");
                }
                let amount = match v {
                    Some(a) => a,
                    None => 1,
                };
                self.target = (self.target + amount).min(self.max_duty_cycle_index);
            }
            A::Down(v) => {
                if !self.available_actions.contains(&Action::Down(None)) {
                    return Err("Action not available for device.");
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
                    return Err("Action not available for device.");
                }
                self.target = 1;
            }
            A::Max => {
                if !self.available_actions.contains(&action) {
                    return Err("Action not available for device.");
                }
                self.target = self.max_duty_cycle_index;
            }
            A::Reverse => {
                if !self.available_actions.contains(&action) {
                    return Err("Action not available for device.");
                }
                self.reversed = !self.reversed;
            }
            A::Set(v) => {
                if !self.available_actions.contains(&Action::Set(0)) {
                    return Err("Action not available for device.");
                }
                if v > self.max_duty_cycle_index {
                    return Err("You attempted to set the target, to something larger than the max duty cycle index");
                }
                self.target = v.min(self.max_duty_cycle_index);
            }
        }
        self.action = action;
        self.updated = true;
        Ok(())
    }

    pub fn needs_hardware_duty_cycle_update(&self) -> bool {
        self.updated
    }

    /// Gets the updated duty cycle
    ///
    /// Given the devices 'duty_cycle's, get the 'target's duty cycle
    /// Said duty cycle is scaled by 'max_duty_cycle' which is to be entered
    /// as a percent where the max is , so 0 through 100 incluseve.

    // TODO: needs testing
    pub fn get_and_update_duty_cycle(&mut self, max_duty_cycle: &u32) -> u32 {
        let ds = match self.duty_cycles[self.target] {
            Some(ds) => ds,
            None => self.duty_cycles[self.max_duty_cycle_index].expect("Something went very wrong! Somehow self.max_duty_cycle_index is larger than the index of the last Some value in self.duty_cycles.")
        };
        self.updated = false;
        ds * max_duty_cycle / 100
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
            devices: Arc::clone(&self.devices),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_synonyms_count() {
        use std::mem;
        assert_eq!(mem::variant_count::<Action>(), ACTION_SYNONYMS.len());
    }

    #[test]
    fn action_same_variant() {
        let one = Action::On;
        let two = Action::On;
        assert!(one.same_variant(&two));

        let three = Action::Up(None);
        let four = Action::Up(Some(3));
        assert!(three.same_variant(&four));

        let five = Action::Set(2);
        let six = Action::Set(6);
        assert!(five.same_variant(&six));

        assert!(!three.same_variant(&six));
    }

    #[test]
    fn action_from_str() {
        let text = "on";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::On));

        let text = "off";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Off));

        let text = "up";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Up(Some(5))));

        let text = "up";
        let val = None;
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Up(None)));

        let text = "down";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Down(Some(5))));

        let text = "down";
        let val = None;
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Down(None)));

        let text = "minimum";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Min));

        let text = "maximum";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Max));

        let text = "reverse";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Reverse));

        let text = "set";
        let val = Some(5);
        let actual = Action::from_str(text, val);
        assert_eq!(actual, Ok(Action::Set(5)));

        let text = "set";
        let val = None;
        let actual = Action::from_str(text, val);
        assert!(actual.is_err());

        let text = "not_anything";
        let val = None;
        let actual = Action::from_str(text, val);
        assert!(actual.is_err());
    }

    #[test]
    fn action_from_uuid() {
        let uuid = 0x928e9b929939486b998d69613f89a9a6;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::On));

        let uuid = 0x13df417d74d2443b87e3de60557b75b8;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Off));

        let uuid = 0xbc6c6eeba0ba40e0a57ff5186d4350ce;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Up(Some(5))));

        let uuid = 0xbc6c6eeba0ba40e0a57ff5186d4350ce;
        let val = None;
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Up(None)));

        let uuid = 0x62865402c86245eea282d4f2ca8fd51b;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Down(Some(5))));

        let uuid = 0x62865402c86245eea282d4f2ca8fd51b;
        let val = None;
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Down(None)));

        let uuid = 0x4aad1b26ea9b455190d0d917102b7f36;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Min));

        let uuid = 0x4ffb631fa4ba4fb5a189f7a3bb9dfa01;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Max));

        let uuid = 0x1a8a1df0523e4acb8390b872329a9ca7;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Reverse));

        let uuid = 0x2a4fae8107134e1fa8187ac56e4f13e4;
        let val = Some(5);
        let actual = Action::from_u128(uuid, val);
        assert_eq!(actual, Ok(Action::Set(5)));

        let uuid = 0x2a4fae8107134e1fa8187ac56e4f13e4;
        let val = None;
        let actual = Action::from_u128(uuid, val);
        assert!(actual.is_err());

        let uuid = 0x1234;
        let val = None;
        let actual = Action::from_u128(uuid, val);
        assert!(actual.is_err());
    }

    #[test]
    fn action_to_str() {
        assert_eq!(Action::On.to_str(), "on");

        assert_eq!(Action::Off.to_str(), "off");

        assert_eq!(Action::Up(Some(3)).to_str(), "up");

        assert_eq!(Action::Up(None).to_str(), "up");

        assert_eq!(Action::Down(Some(2)).to_str(), "down");

        assert_eq!(Action::Min.to_str(), "minimum");

        assert_eq!(Action::Max.to_str(), "maximum");

        assert_eq!(Action::Reverse.to_str(), "reverse");

        assert_eq!(Action::Set(3).to_str(), "set");
    }

    #[test]
    fn action_to_uuid() {
        assert_eq!(
            Action::On.to_uuid(),
            Uuid::from_u128(0x928e9b929939486b998d69613f89a9a6)
        );

        assert_eq!(
            Action::Off.to_uuid(),
            Uuid::from_u128(0x13df417d74d2443b87e3de60557b75b8)
        );

        assert_eq!(
            Action::Up(Some(3)).to_uuid(),
            Uuid::from_u128(0xbc6c6eeba0ba40e0a57ff5186d4350ce)
        );

        assert_eq!(
            Action::Up(None).to_uuid(),
            Uuid::from_u128(0xbc6c6eeba0ba40e0a57ff5186d4350ce)
        );

        assert_eq!(
            Action::Down(Some(3)).to_uuid(),
            Uuid::from_u128(0x62865402c86245eea282d4f2ca8fd51b)
        );

        assert_eq!(
            Action::Down(None).to_uuid(),
            Uuid::from_u128(0x62865402c86245eea282d4f2ca8fd51b)
        );

        assert_eq!(
            Action::Min.to_uuid(),
            Uuid::from_u128(0x4aad1b26ea9b455190d0d917102b7f36)
        );

        assert_eq!(
            Action::Max.to_uuid(),
            Uuid::from_u128(0x4ffb631fa4ba4fb5a189f7a3bb9dfa01)
        );

        assert_eq!(
            Action::Reverse.to_uuid(),
            Uuid::from_u128(0x1a8a1df0523e4acb8390b872329a9ca7)
        );

        assert_eq!(
            Action::Set(3).to_uuid(),
            Uuid::from_u128(0x2a4fae8107134e1fa8187ac56e4f13e4)
        );
    }

    #[test]
    fn action_get_value() {
        let actual = Action::On.get_value();
        let expected = None;
        assert_eq!(actual, expected);

        let actual = Action::Up(Some(2)).get_value();
        let expected = Some(2);
        assert_eq!(actual, expected);

        let actual = Action::Up(None).get_value();
        let expected = None;
        assert_eq!(actual, expected);

        let actual = Action::Down(Some(2)).get_value();
        let expected = Some(2);
        assert_eq!(actual, expected);

        let actual = Action::Down(None).get_value();
        let expected = None;
        assert_eq!(actual, expected);

        let actual = Action::Set(4).get_value();
        let expected = Some(4);
        assert_eq!(actual, expected);
    }

    #[test]
    fn device_new() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string()).unwrap();
        assert_eq!(device.uuid, Uuid::from_u128(0x12345));
        assert_eq!(device.name, String::from("name"));
        assert_eq!(device.action, Action::Off);
        assert_eq!(
            device.available_actions,
            Vec::from([
                Action::On,
                Action::Off,
                Action::Up(None),
                Action::Down(None),
                Action::Min,
                Action::Max,
                Action::Set(0)
            ])
        );
        assert_eq!(device.default_target, 3);
        assert_eq!(
            device.duty_cycles,
            [
                Some(0),
                Some(2),
                Some(4),
                Some(8),
                Some(16),
                Some(32),
                Some(64),
                Some(96)
            ]
        );
        assert_eq!(device.max_duty_cycle_index, 7);
        assert_eq!(device.target, 0);
        assert_eq!(device.freq_Hz, 100);
        assert_eq!(device.device_group, None);
        assert_eq!(device.reversed, false);
        assert_eq!(device.updated, true);
    }

    #[test]
    fn device_action() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .action(Action::Set(4))
            .unwrap();
        assert_eq!(device.action, Action::Set(4));
    }

    #[test]
    fn device_available_actions() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .available_actions(vec![Action::On, Action::Up(None), Action::Set(0)])
            .unwrap();
        assert_eq!(
            device.available_actions,
            vec![Action::On, Action::Up(None), Action::Set(0)]
        );
    }

    #[test]
    //#[should_panic(expected = "If Action::Up is an an available_action")]
    fn device_available_actions_panic_up() {
        // should panic if Up, Down, and Set don't have the right values
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .available_actions(vec![Action::On, Action::Up(Some(0))]);
        assert!(device.is_err());
    }

    #[test]
    //#[should_panic]
    fn device_available_actions_panic_down() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .available_actions(vec![Action::On, Action::Down(Some(0))]);
        assert!(device.is_err());
    }

    #[test]
    //#[should_panic]
    fn device_available_actions_panic_set() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .available_actions(vec![Action::On, Action::Set(1)]);
        assert!(device.is_err());
    }

    #[test]
    fn device_get_available_actions() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .available_actions(vec![Action::On, Action::Set(0)])
            .unwrap();

        assert_eq!(
            device.get_available_actions(),
            &vec![Action::On, Action::Set(0)]
        );
    }

    #[test]
    fn device_default_target() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .default_target(5)
            .unwrap();
        assert_eq!(device.default_target, 5);
    }

    #[test]
    fn device_default_target_panic() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .duty_cycles([Some(0), Some(1), Some(3), Some(4), None, None, None, None])
            .unwrap()
            .default_target(5);
        assert!(device.is_err());
    }

    #[test]
    fn device_get_default_target() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .default_target(5)
            .unwrap();

        assert_eq!(device.get_default_target(), 5);
    }

    #[test]
    fn device_duty_cycles() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .duty_cycles([Some(0), Some(1), Some(3), Some(4), None, None, None, None])
            .unwrap();
        assert_eq!(device.max_duty_cycle_index, 3);
    }

    #[test]
    //#[should_panic]
    fn device_duty_cycles_default_target_panic() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .duty_cycles([Some(0), Some(1), None, None, None, None, None, None]);
        assert!(device.is_err());
    }

    #[test]
    //#[should_panic]
    fn device_duty_cycles_target_panic() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .target(6)
            .unwrap()
            .duty_cycles([Some(0), Some(1), Some(3), Some(4), None, None, None, None]);
        assert!(device.is_err());
    }

    #[test]
    fn device_get_duty_cycles() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .duty_cycles([Some(0), Some(1), Some(3), Some(4), None, None, None, None])
            .unwrap();
        assert_eq!(
            device.get_duty_cycles(),
            &[Some(0), Some(1), Some(3), Some(4), None, None, None, None]
        );
    }

    #[test]
    fn device_target() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .duty_cycles([Some(0), Some(1), Some(3), Some(4), None, None, None, None])
            .unwrap()
            .target(2)
            .unwrap();
        assert_eq!(device.target, 2);
    }

    #[test]
    //#[should_panic]
    fn device_target_panic() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .duty_cycles([Some(0), Some(1), Some(3), Some(4), None, None, None, None])
            .unwrap()
            .target(6);
        assert!(device.is_err());
    }

    #[test]
    fn device_freq_hz() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .freq_Hz(88)
            .unwrap();
        assert_eq!(device.freq_Hz, 88);
    }

    #[test]
    fn device_device_group_some() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .device_group(Some(DeviceGroup::Light))
            .unwrap();
        assert_eq!(device.device_group, Some(DeviceGroup::Light));
    }

    #[test]
    fn device_device_group_none() {
        let device = Device::build(Uuid::from_u128(0x12345), "name".to_string())
            .unwrap()
            .device_group(None)
            .unwrap();
        assert_eq!(device.device_group, None);
    }

    #[test]
    fn device_to_json() {
        let device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .action(Action::Up(Some(3)))
        .unwrap();

        let jsoned = device.to_json();

        let actual = "{\"uuid\":\"f1d34301-c916-42a8-8c7c-274828177649\",\"name\":\"Device1\",\"action\":{\"Up\":3},\"available_actions\":[\"On\",\"Off\",{\"Up\":null},{\"Down\":null},\"Min\",\"Max\",{\"Set\":0}],\"default_target\":3,\"duty_cycles\":[0,2,4,8,16,32,64,96],\"max_duty_cycle_index\":7,\"target\":0,\"freq_Hz\":100,\"device_group\":null,\"reversed\":false,\"updated\":true}";

        assert_eq!(jsoned, actual);
    }

    #[test]
    fn device_from_json() {
        let device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .action(Action::Up(Some(3)))
        .unwrap();

        let json_text = "{\"uuid\":\"f1d34301-c916-42a8-8c7c-274828177649\",\"name\":\"Device1\",\"action\":{\"Up\":3},\"available_actions\":[\"On\",\"Off\",{\"Up\":null},{\"Down\":null},\"Min\",\"Max\",{\"Set\":0}],\"default_target\":3,\"duty_cycles\":[0,2,4,8,16,32,64,96],\"max_duty_cycle_index\":7,\"target\":0,\"freq_Hz\":100,\"device_group\":null,\"reversed\":false,\"updated\":true}";

        let actual = Device::from_json(&json_text.to_string());

        assert_eq!(device, actual.unwrap());
    }

    #[test]
    fn device_take_action_action_missing() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .available_actions(vec![])
        .unwrap()
        .target(2)
        .unwrap();

        let err = device.take_action(On);

        assert!(err.is_err());
    }

    #[test]
    fn device_take_action_on() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(2)
        .unwrap();

        let _ = device.take_action(On);

        assert_eq!(device.target, 3);
        assert_eq!(device.get_and_update_duty_cycle(&255), 8 * 255 / 100);
        assert_eq!(device.action, On);
    }

    #[test]
    fn device_take_action_off() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(2)
        .unwrap()
        .action(On)
        .unwrap();

        let _ = device.take_action(Off);

        assert_eq!(device.target, 0);
        assert_eq!(device.get_and_update_duty_cycle(&255), 0);
        assert_eq!(device.action, Off);
    }

    #[test]
    fn device_take_action_up_none() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(2)
        .unwrap();

        let _ = device.take_action(Up(None));

        assert_eq!(device.target, 3);
        assert_eq!(device.get_and_update_duty_cycle(&255), 8 * 255 / 100);
        assert_eq!(device.action, Up(None));
    }

    #[test]
    fn device_take_action_up_some() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(2)
        .unwrap();

        let _ = device.take_action(Up(Some(2)));

        assert_eq!(device.target, 4);
        assert_eq!(device.get_and_update_duty_cycle(&255), 16 * 255 / 100);
        assert_eq!(device.action, Up(Some(2)));
    }

    #[test]
    fn device_take_action_up_already_max() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(7)
        .unwrap();

        let _ = device.take_action(Up(None));

        assert_eq!(device.target, 7);
        assert_eq!(device.get_and_update_duty_cycle(&255), 96 * 255 / 100);
        assert_eq!(device.action, Up(None));
    }

    #[test]
    fn device_take_action_down_none() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(2)
        .unwrap();

        let _ = device.take_action(Down(None));

        assert_eq!(device.target, 1);
        assert_eq!(device.get_and_update_duty_cycle(&255), 2 * 255 / 100);
        assert_eq!(device.action, Down(None));
    }

    #[test]
    fn device_take_action_down_some() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(2)
        .unwrap();

        let _ = device.take_action(Down(Some(2)));

        assert_eq!(device.target, 0);
        assert_eq!(device.get_and_update_duty_cycle(&255), 0);
        assert_eq!(device.action, Down(Some(2)));
    }

    #[test]
    fn device_take_action_down_already_off() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(0)
        .unwrap();

        let _ = device.take_action(Down(None));

        assert_eq!(device.target, 0);
        assert_eq!(device.get_and_update_duty_cycle(&255), 0);
        assert_eq!(device.action, Down(None));
    }

    #[test]
    fn device_take_action_min() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(5)
        .unwrap();

        let _ = device.take_action(Min);

        assert_eq!(device.target, 1);
        assert_eq!(device.get_and_update_duty_cycle(&255), 2 * 255 / 100);
        assert_eq!(device.action, Min);
    }

    #[test]
    fn device_take_action_max() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(5)
        .unwrap();

        let _ = device.take_action(Max);

        assert_eq!(device.target, 7);
        assert_eq!(device.get_and_update_duty_cycle(&255), 96 * 255 / 100);
        assert_eq!(device.action, Max);
    }

    #[test]
    fn device_take_action_reverse() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .available_actions(vec![On, Off, Reverse])
        .unwrap();

        let _ = device.take_action(Reverse);
        device.reversed = !device.reversed;
        assert!(!device.reversed);
        assert_eq!(device.action, Reverse);
    }

    #[test]
    fn device_take_action_set() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap();

        let _ = device.take_action(Set(3));
        assert_eq!(device.target, 3);
        assert_eq!(device.get_and_update_duty_cycle(&255), 8 * 255 / 100);
        assert_eq!(device.action, Set(3));

        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .duty_cycles([Some(0), Some(1), Some(3), Some(4), None, None, None, None])
        .unwrap();

        let output = device.take_action(Set(5));
        assert!(output.is_err());
    }

    #[test]
    fn device_take_action_get_and_update_duty_cycle() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(3)
        .unwrap();

        assert_eq!(device.get_and_update_duty_cycle(&255), 8 * 255 / 100);
    }

    #[test]
    fn device_needs_hardware_duty_cycle_update() {
        use Action::*;
        let mut device = Device::build(
            Uuid::from_u128(0xf1d34301c91642a88c7c274828177649),
            String::from("Device1"),
        )
        .unwrap()
        .target(3)
        .unwrap();

        assert!(device.needs_hardware_duty_cycle_update());

        device.get_and_update_duty_cycle(&255);

        assert!(!device.needs_hardware_duty_cycle_update());

        let _ = device.take_action(On);

        assert!(device.needs_hardware_duty_cycle_update());
    }

    #[test]
    fn devices_append() {
        let mut lights1 = Devices {
            devices: Arc::new(Mutex::new(Vec::from([
                Device::build(
                    Uuid::from_u128(0x584507902e74f44b67902b90775abda),
                    "bedroom light".to_string(),
                )
                .unwrap(),
                Device::build(
                    Uuid::from_u128(0x36bc0fe1b00742809ec6b36c8bc98537),
                    "kitchen light".to_string(),
                )
                .unwrap(),
            ]))),
        };
        let mut lights2 = Devices {
            devices: Arc::new(Mutex::new(Vec::from([
                Device::build(
                    Uuid::from_u128(0xad87d775f9fd4bc29f06c47937f6df4a),
                    "counter light".to_string(),
                )
                .unwrap(),
                Device::build(
                    Uuid::from_u128(0xc252b58ab7f046fc9fda00f9947904df),
                    "outside light".to_string(),
                )
                .unwrap(),
            ]))),
        };
        lights1.append(&mut lights2);

        assert_eq!(lights1.devices.lock().unwrap().len(), 4);

        let names = lights1
            .devices
            .lock()
            .unwrap()
            .iter()
            .map(|d| d.name.clone())
            .collect::<Vec<String>>();
        assert!(names.contains(&"bedroom light".to_string()));
        assert!(names.contains(&"kitchen light".to_string()));
        assert!(names.contains(&"counter light".to_string()));
        assert!(names.contains(&"outside light".to_string()));
    }
}
