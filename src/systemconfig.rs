
use stm32f4xx_hal::pac::Peripherals;
use stm32f4xx_hal::flash::{FlashExt, LockedFlash, UnlockedFlash};
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
//use byteorder::ByteOrder;


use crate::watering_logic::{PlantData, WaterConfig};

//use nucleo_f401re::{
//    SerialInterface
//}




pub struct SystemConfig
{
    storage : LockedFlash,
}

const storage_identifier    : u32 = 0x11223344;
const storage_version       : u32 = 1;
// we use Sector 7   0x0806 0000 - 0x0807 FFFF  128 Kbytes
const sector : u8 = 7;
//const startaddr : usize = 0x0806_0000;
const startaddr : u32 = 0x6_0000;

struct StoredSettings 
{
    struct_identifier : u32,// Identifier struct 
    struct_version    : u32,// version 
    plant_data        : PlantData, // data 
    // crc 
}

impl StoredSettings
{
    fn new() -> StoredSettings 
    {
        StoredSettings 
        {
            struct_identifier : 0, // only succefull read fills it 
            struct_version : storage_version,
            plant_data : [  WaterConfig::new(), 
                            WaterConfig::new(), 
                            WaterConfig::new(), 
                            WaterConfig::new(), 
                            WaterConfig::new(), 
                            WaterConfig::new()
                        ],
        }
    }

    fn to_array(&mut self) -> [u8; 14]
    {
        let mut settings :[u8; 14] = [ 0; 14 ];
        settings[0] = ((self.struct_identifier  >> 24) & 0xff) as u8;
        settings[1] = ((self.struct_identifier  >> 16) & 0xff) as u8;
        settings[2] = ((self.struct_identifier  >> 8) & 0xff) as u8;
        settings[3] = ((self.struct_identifier  >> 0) & 0xff) as u8;

        settings[4] = ((self.struct_version  >> 24) & 0xff) as u8;
        settings[5] = ((self.struct_version  >> 16) & 0xff) as u8;
        settings[6] = ((self.struct_version  >> 8) & 0xff) as u8;
        settings[7] = ((self.struct_version  >> 0) & 0xff) as u8;

        let mut writeindex = 8;

        for item in self.plant_data[0].to_array()
        {
            settings[writeindex] = item; 
            writeindex += 1;
        }

        return settings;
    }
}

impl<'a> IntoIterator for &'a StoredSettings
{
    type Item = u8;
    type IntoIter = StoredSettingsIntoIterator<'a>;

    // fn next(&self) -> Option<Self::u32>
    // {
    //     Some()
    // }

    fn into_iter(self) -> Self::IntoIter
    {
        StoredSettingsIntoIterator { data : self, index : 0 }
    }
}

pub struct StoredSettingsIntoIterator<'a> 
{
    data: &'a StoredSettings,
    index: usize,
}

impl<'a> Iterator for StoredSettingsIntoIterator<'a>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8>
    {
        let result = match self.index 
        {
            0 => ((self.data.struct_identifier  >> 24) & 0xff) as u8,
            1 => ((self.data.struct_identifier  >> 16) & 0xff) as u8,
            2 => ((self.data.struct_identifier  >> 8) & 0xff) as u8,
            3 => ((self.data.struct_identifier  >> 0) & 0xff) as u8,
            4 => ((self.data.struct_version  >> 24) & 0xff) as u8,
            5 => ((self.data.struct_version  >> 16) & 0xff) as u8,
            6 => ((self.data.struct_version  >> 8) & 0xff) as u8,
            7 => ((self.data.struct_version  >> 0) & 0xff) as u8,
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }

}

impl SystemConfig 
{
    pub fn new(storage : LockedFlash) -> Self 
    {
        Self {storage}
    }


    pub fn safe_config(&mut self, config : PlantData)
    {
        let mut progamm_data : StoredSettings = { StoredSettings {
            struct_identifier : storage_identifier,
            struct_version : storage_version,
            plant_data : config,
        } };

        let mut unlocked_flash = self.storage.unlocked();

        NorFlash::erase(&mut unlocked_flash, startaddr, 0x2_0000).unwrap(); // sector is 128k big 
        NorFlash::write(&mut unlocked_flash, startaddr, &progamm_data.to_array()).unwrap();
    
        // Lock flash by dropping
        drop(unlocked_flash);

    }
    
    pub fn read_config(&mut self) -> PlantData
    {
        let mut read_settings : StoredSettings = StoredSettings::new();
        let mut read_bytes : [u8; 14] = [0; 14];

        ReadNorFlash::read(&mut self.storage, startaddr, &mut read_bytes).unwrap();

        //read_settings.struct_identifier = (read_bytes[0] as u32) << 24 | (read_bytes[1] as u32) << 16 | (read_bytes[2] as u32) << 8 | (read_bytes[3] as u32) << 0;
        read_settings.struct_identifier = read_bytes[0] as u32;
        read_settings.struct_identifier <<=8; 
        read_settings.struct_identifier |= read_bytes[1] as u32;
        read_settings.struct_identifier <<=8; 
        read_settings.struct_identifier |= read_bytes[2] as u32;
        read_settings.struct_identifier <<=8; 
        read_settings.struct_identifier |= read_bytes[3] as u32;

        //read_settings.struct_version    = (read_bytes[4] as u32) << 24 + (read_bytes[5] as u32) << 16 + (read_bytes[6] as u32) << 8 + (read_bytes[7] as u32) << 0;
        read_settings.struct_version    = read_bytes[4] as u32;
        read_settings.struct_version    <<= 8;
        read_settings.struct_version    |= read_bytes[5] as u32;
        read_settings.struct_version    <<= 8;
        read_settings.struct_version    |= read_bytes[6] as u32;
        read_settings.struct_version    <<= 8;
        read_settings.struct_version    |= read_bytes[7] as u32;

        if (read_settings.struct_identifier == storage_identifier) && (read_settings.struct_version == storage_version)
        {
            // Fill 
            read_settings.plant_data = [
                WaterConfig::new_ex(
                    (read_bytes[8] as u16) << 8 + (read_bytes[9] as u16), 
                    (read_bytes[10] as u16) << 8 + (read_bytes[11] as u16), 
                    (read_bytes[12] as u16) << 8 + (read_bytes[13] as u16),),
                WaterConfig::new(),
                WaterConfig::new(),
                WaterConfig::new(),
                WaterConfig::new(),
                WaterConfig::new(),
            ];   
        }

        return read_settings.plant_data;
    }
}
