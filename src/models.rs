use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VehicleData {
    pub waybill_id: String,
    pub service_type: String,
    pub vehicle_no: String,
    pub schedule_no: String,
    pub last_updated: Option<DateTime<Utc>>,
    pub duty_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleServiceTypeResponse {
    pub vehicle_no: String,
    pub service_type: String,
    pub waybill_id: Option<String>,
    pub schedule_no: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatLong {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NandiStop {
    pub id: String,
    pub code: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NandiTrip {
    pub id: String,
    pub direction: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NandiPattern {
    pub id: String,
    pub desc: String,
    #[serde(rename = "routeId")]
    pub route_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NandiPatternDetails {
    pub id: String,
    pub desc: Option<String>,
    #[serde(rename = "routeId")]
    pub route_id: String,
    pub stops: Vec<NandiStop>,
    pub trips: Vec<NandiTrip>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NandiRoutesRes {
    pub id: String,
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    #[serde(rename = "longName")]
    pub long_name: Option<String>,
    pub mode: String,
    #[serde(rename = "agencyName")]
    pub agency_name: Option<String>,
    #[serde(rename = "tripCount")]
    pub trip_count: Option<i32>,
    #[serde(rename = "stopCount")]
    pub stop_count: Option<i32>,
    #[serde(rename = "startPoint")]
    pub start_point: Option<LatLong>,
    #[serde(rename = "endPoint")]
    pub end_point: Option<LatLong>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStopMapping {
    #[serde(rename = "estimatedTravelTimeFromPreviousStop")]
    pub estimated_travel_time_from_previous_stop: Option<i32>,
    #[serde(rename = "providerCode")]
    pub provider_code: String,
    #[serde(rename = "routeCode")]
    pub route_code: String,
    #[serde(rename = "sequenceNum")]
    pub sequence_num: i32,
    #[serde(rename = "stopCode")]
    pub stop_code: String,
    #[serde(rename = "stopName")]
    pub stop_name: String,
    #[serde(rename = "stopPoint")]
    pub stop_point: LatLong,
    #[serde(rename = "vehicleType")]
    pub vehicle_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStopMappingWithGeojson {
    #[serde(rename = "estimatedTravelTimeFromPreviousStop")]
    pub estimated_travel_time_from_previous_stop: Option<i32>,
    #[serde(rename = "providerCode")]
    pub provider_code: String,
    #[serde(rename = "routeCode")]
    pub route_code: String,
    #[serde(rename = "sequenceNum")]
    pub sequence_num: i32,
    #[serde(rename = "stopCode")]
    pub stop_code: String,
    #[serde(rename = "stopName")]
    pub stop_name: String,
    #[serde(rename = "stopPoint")]
    pub stop_point: LatLong,
    #[serde(rename = "vehicleType")]
    pub vehicle_type: String,
    #[serde(rename = "stopGeojson")]
    pub stop_geojson: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GTFSStop {
    pub id: String,
    pub code: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    #[serde(rename = "stationId")]
    pub station_id: Option<String>,
    pub cluster: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopGeojson {
    pub stop_code: String,
    pub gtfs_id: String,
    pub geo_json: String,
    pub gates: String,
}

#[derive(Debug, Clone, Default)]
pub struct GTFSRouteData {
    pub mappings: Vec<Arc<RouteStopMapping>>,
    pub by_route: HashMap<String, Vec<usize>>,
    pub by_stop: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Clone, Default)]
pub struct GTFSData {
    pub routes_by_gtfs: HashMap<String, HashMap<String, NandiRoutesRes>>,
    pub route_data_by_gtfs: HashMap<String, GTFSRouteData>,
    pub children_by_parent: HashMap<String, HashMap<String, Vec<String>>>,
    pub data_hash: HashMap<String, String>,
    pub stop_geojsons: HashMap<String, StopGeojson>,
}

impl GTFSData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_data(&mut self, temp_data: GTFSData) {
        self.routes_by_gtfs = temp_data.routes_by_gtfs;
        self.route_data_by_gtfs = temp_data.route_data_by_gtfs;
        self.children_by_parent = temp_data.children_by_parent;
        self.data_hash = temp_data.data_hash;
        self.stop_geojsons = temp_data.stop_geojsons;
    }
}

pub fn cast_vehicle_type(vehicle_type: &str) -> String {
    if vehicle_type == "RAIL" {
        "METRO".to_string()
    } else {
        vehicle_type.to_string()
    }
}

pub fn clean_identifier(identifier: &str) -> String {
    // URL decode and remove GTFS ID prefix if present
    let decoded = urlencoding::decode(identifier).unwrap_or_else(|_| identifier.to_string().into());

    // Remove GTFS ID prefix if present (format: gtfs_id:code)
    decoded.split(':').last().unwrap_or(&decoded).to_string()
}
