use micro_rdk::common::config::ConfigType;
use micro_rdk::common::registry::{ComponentRegistry, Dependency, RegistryError};
use micro_rdk::common::status::{Status, StatusError};
use micro_rdk::DoCommand;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use micro_rdk::common::sensor::{
    GenericReadingsResult, Readings, Sensor, SensorError, SensorResult, SensorT, SensorType,
    TypedReadingsResult,
};

use micro_rdk::esp32::esp_idf_svc::hal::gpio::{AnyIOPin, Input, PinDriver, Pull};

pub fn register_models(registry: &mut ComponentRegistry) -> Result<(), RegistryError> {
    registry.register_sensor("hw201", &HW201Sensor::from_config)
}

#[derive(DoCommand)]
pub struct HW201Sensor {
    pin: Esp32GPIOPin,
}

pub struct Esp32GPIOPin {
    pin: i32,
    driver: PinDriver<'static, AnyIOPin, Input>,
}

impl Esp32GPIOPin {
    pub fn new(pin: i32, pull: Option<Pull>) -> Result<Self, SensorError> {
        let mut driver = PinDriver::input(unsafe { AnyIOPin::new(pin) })
            .map_err(|_e| SensorError::SensorGenericError("Failed to create pin driver"))?;
        if let Some(pull) = pull {
            driver
                .set_pull(pull)
                .map_err(|_e| SensorError::SensorGenericError("Failed to set pull"))?;
        }
        Ok(Self { pin, driver })
    }

    pub fn pin(&self) -> i32 {
        self.pin
    }

    pub fn is_low(&self) -> bool {
        self.driver.is_low()
    }

    pub fn is_high(&self) -> bool {
        self.driver.is_high()
    }
}

impl HW201Sensor {
    pub fn from_config(cfg: ConfigType, deps: Vec<Dependency>) -> Result<SensorType, SensorError> {
        let result = cfg.get_attribute::<i32>("pin");
        if result.is_err() {
            return Err(SensorError::SensorGenericError("Pin not found"));
        }
        let pin_number = result.unwrap();
        let pin = Esp32GPIOPin::new(pin_number, Some(Pull::Up))?;
        Ok(Arc::new(Mutex::new(HW201Sensor { pin })))
    }
}

impl Sensor for HW201Sensor {}

impl Readings for HW201Sensor {
    fn get_generic_readings(&mut self) -> Result<GenericReadingsResult, SensorError> {
        Ok(self
            .get_readings()?
            .into_iter()
            .map(|v| {
                (
                    v.0,
                    SensorResult::<f64> {
                        value: if v.1 { 1.0 } else { 0.0 },
                    }
                    .into(),
                )
            })
            .collect())
    }
}

impl SensorT<bool> for HW201Sensor {
    fn get_readings(&self) -> Result<TypedReadingsResult<bool>, SensorError> {
        let mut x = HashMap::new();
        x.insert("pin_value".to_string(), self.pin.is_low());
        Ok(x)
    }
}

impl Status for HW201Sensor {
    fn get_status(&self) -> Result<Option<micro_rdk::google::protobuf::Struct>, StatusError> {
        Ok(Some(micro_rdk::google::protobuf::Struct {
            fields: HashMap::new(),
        }))
    }
}
