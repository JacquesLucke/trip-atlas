#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsData {
    pub agencies: Vec<GtfsAgency>,
    pub calendars: Vec<GtfsCalendar>,
    pub calendar_dates: Vec<GtfsCalendarDate>,
    pub routes: Vec<GtfsRoute>,
    pub stops: Vec<GtfsStop>,
    pub stop_times: Vec<GtfsStopTime>,
    pub trips: Vec<GtfsTrip>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsStop {
    pub id: String,
    pub code: Option<String>,
    pub name: Option<String>,
    pub parent_station_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsStopTime {
    pub arrival_time: Option<u32>,
    pub departure_time: Option<u32>,
    pub stop_id: String,
    pub stop_sequence: u16,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsTrip {
    pub id: String,
    pub service_id: String,
    pub route_id: String,
    pub short_name: Option<String>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsRoute {
    pub id: String,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub route_type: GtfsRouteType,
    pub agency_id: Option<String>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub enum GtfsRouteType {
    Tramway,
    Subway,
    Rail,
    Bus,
    Ferry,
    CableCar,
    Gondola,
    Funicular,
    Coach,
    Air,
    Taxi,
    Other(i16),
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsAgency {
    pub id: Option<String>,
    pub name: String,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsCalendar {
    pub id: String,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
    pub start_date: String,
    pub end_date: String,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct GtfsCalendarDate {
    pub service_id: String,
    pub date: String,
    pub exception_type: GtfsExceptionType,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub enum GtfsExceptionType {
    Added,
    Deleted,
}
