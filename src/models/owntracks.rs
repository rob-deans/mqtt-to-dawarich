use serde::{Deserialize, Serialize};

// Only for _type = location
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwntracksPayload {
    pub _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<String>, // Android

    // Location-related fields
    pub lat: f64, // Required for both iOS and Android
    pub lon: f64, // Required for both iOS and Android
    pub tst: i64, // Required for both iOS and Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc: Option<i32>, // iOS, Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<i32>, // iOS, Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cog: Option<u16>, // iOS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vel: Option<i32>, // iOS, Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vac: Option<i32>, // iOS

    // Device-related fields
    pub bs: u8, // Required for both iOS and Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batt: Option<i32>, // iOS, Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn: Option<String>, // iOS, Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i32>, // iOS, Android

    // Additional tracking fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tid: Option<String>, // Required for both iOS and Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m: Option<u8>, // iOS, Android (monitoring mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>, // iOS

    // Extended data fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<f32>, // iOS (barometric pressure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poi: Option<String>, // iOS (point of interest name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>, // iOS (Base64 encoded image associated with the poi)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imagename: Option<String>, // iOS (Name of the image associated with the poi)

    // Region-related fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inregions: Option<Vec<String>>, // iOS, Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inrids: Option<Vec<String>>, // iOS, Android

    // Network-related fields
    #[serde(skip_serializing_if = "Option::is_none", rename = "SSID")]
    pub ssid: Option<String>, // iOS
    #[serde(skip_serializing_if = "Option::is_none", rename = "BSSID")]
    pub bssid: Option<String>, // iOS

    // Miscellaneous fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>, // iOS, Android >= 2.4
}

impl PartialEq for OwntracksPayload {
    fn eq(&self, other: &Self) -> bool {
        self._id == other._id
    }
}
