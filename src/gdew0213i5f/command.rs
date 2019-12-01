//! SPI Commands for the GDEW0213I5F 2.13" E-Ink Display

use crate::traits;

/// GDEW0213I5F commands.
///
/// For data arguments and default values for the commands see the datasheet.
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Command {
    PANEL_SETTING = 0x00,

    POWER_SETTING = 0x01,

    POWER_OFF = 0x02,

    POWER_OFF_SEQUENCE_SETTING = 0x03,

    POWER_ON = 0x04,

    POWER_ON_MEASURE = 0x05,

    BOOSTER_SOFT_START = 0x06,

    DEEP_SLEEP = 0x07,

    DISPLAY_START_TRANSMISSION_1 = 0x10,

    DATA_STOP = 0x11,

    DISPLAY_REFRESH = 0x12,

    DISPLAY_START_TRANSMISSION_2 = 0x13,

    AUTO_SEQUENCE = 0x17,

    VCOM_LUT = 0x20,

    W2W_LUT = 0x21,

    B2W_LUT = 0x22,

    W2B_LUT = 0x23,

    B2B_LUT = 0x24,

    LUT_OPTION = 0x2a,

    PLL_CONTROL = 0x30,

    TEMPERATURE_SENSOR_CALIBRATION = 0x40,

    TEMPERATURE_SENSOR_SELECTION = 0x41,

    TEMPERATURE_SENSOR_WRITE = 0x42,

    TEMPERATURE_SENSOR_READ = 0x43,

    PANEL_BREAK_CHECK = 0x44,

    VCOM_AND_DATA_INTERVAL_SETTING = 0x50,

    LOW_POWER_DETECTION = 0x51,

    TCON_SETTING = 0x60,

    RESOLUTION_SETTING = 0x61,

    GATE_SOURCE_START_SETTING = 0x65,

    REVISION = 0x70,

    GET_STATUS = 0x71,

    AUTO_MEASUREMENT_VCOM = 0x80,

    READ_VCOM_VALUE = 0x81,

    VCM_DC_SETTING = 0x82,

    PARTIAL_WINDOW = 0x90,

    PARTIAL_IN = 0x91,

    PARTIAL_OUT = 0x92,

    PROGRAM_MODE = 0xa0,

    ACTIVE_PROGRAMMING = 0xa1,

    READ_OTP = 0xa2,

    CASCADE_SETTING = 0xe0,

    POWER_SAVING = 0xe3,

    LVD_VOLTAGE_SELECT = 0xe4,

    FORCE_TEMPERATURE = 0xe5,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::Command;
    use crate::traits::Command as CommandTrait;

    #[test]
    fn command_addr() {
        assert_eq!(Command::BOOSTER_SOFT_START.address(), 0x06);

        assert_eq!(Command::PLL_CONTROL.address(), 0x30);

        assert_eq!(Command::PARTIAL_WINDOW.address(), 0x90);
    }
}
