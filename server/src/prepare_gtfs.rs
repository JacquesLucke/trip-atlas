use std::{io::Write, path::Path};

use crate::gtfs_rkyv::*;
use anyhow::Result;

pub async fn prepare_gtfs(gtfs_folder_path: &Path) -> Result<()> {
    let buffer = gtfs_data_to_rkyv_buffer(gtfs_folder_path)?;
    let output_path = gtfs_folder_path.join("data_rkyv.bin");
    log::info!("Writing data to {:?}", output_path);
    let mut file = std::fs::File::create(&output_path)?;
    file.write_all(&buffer)?;

    Ok(())
}

fn gtfs_data_to_rkyv_buffer(gtfs_folder_path: &Path) -> Result<rkyv::util::AlignedVec> {
    log::info!("Loading original GTFS data from {:?}", gtfs_folder_path);
    let gtfs = gtfs_structures::RawGtfs::from_path(gtfs_folder_path)?;

    log::info!("Preparing stops...");
    let mut gtfs_stops = vec![];
    for stop in gtfs.stops? {
        gtfs_stops.push(GtfsStop {
            id: stop.id.clone(),
            code: stop.code.clone(),
            name: stop.name.clone(),
            parent_station_id: stop.parent_station.clone(),
            latitude: stop.latitude,
            longitude: stop.longitude,
        });
    }

    log::info!("Preparing stop times.");
    let mut gtfs_stop_times = vec![];
    for stop_time in gtfs.stop_times? {
        gtfs_stop_times.push(GtfsStopTime {
            arrival_time: stop_time.arrival_time,
            departure_time: stop_time.departure_time,
            stop_id: stop_time.stop_id.clone(),
            stop_sequence: stop_time.stop_sequence,
        })
    }

    log::info!("Preparing trips.");
    let mut gtfs_trips = vec![];
    for trip in gtfs.trips? {
        gtfs_trips.push(GtfsTrip {
            id: trip.id.clone(),
            service_id: trip.service_id.clone(),
            route_id: trip.route_id.clone(),
            short_name: trip.trip_short_name.clone(),
        });
    }

    log::info!("Preparing routes.");
    let mut gtfs_routes = vec![];
    for route in gtfs.routes? {
        gtfs_routes.push(GtfsRoute {
            id: route.id.clone(),
            short_name: route.short_name.clone(),
            long_name: route.long_name.clone(),
            route_type: match route.route_type {
                gtfs_structures::RouteType::Tramway => GtfsRouteType::Tramway,
                gtfs_structures::RouteType::Subway => GtfsRouteType::Subway,
                gtfs_structures::RouteType::Rail => GtfsRouteType::Rail,
                gtfs_structures::RouteType::Bus => GtfsRouteType::Bus,
                gtfs_structures::RouteType::Ferry => GtfsRouteType::Ferry,
                gtfs_structures::RouteType::CableCar => GtfsRouteType::CableCar,
                gtfs_structures::RouteType::Gondola => GtfsRouteType::Gondola,
                gtfs_structures::RouteType::Funicular => GtfsRouteType::Funicular,
                gtfs_structures::RouteType::Coach => GtfsRouteType::Coach,
                gtfs_structures::RouteType::Air => GtfsRouteType::Air,
                gtfs_structures::RouteType::Taxi => GtfsRouteType::Taxi,
                gtfs_structures::RouteType::Other(other) => GtfsRouteType::Other(other),
            },
            agency_id: route.agency_id.clone(),
        });
    }

    log::info!("Preparing agencies.");
    let mut gtfs_agencies = vec![];
    for agency in gtfs.agencies? {
        gtfs_agencies.push(GtfsAgency {
            id: agency.id.clone(),
            name: agency.name.clone(),
        });
    }

    log::info!("Preparing calendars.");
    let mut gtfs_calendars = vec![];
    for calendar in gtfs.calendar.unwrap()? {
        gtfs_calendars.push(GtfsCalendar {
            id: calendar.id.clone(),
            monday: calendar.monday,
            tuesday: calendar.tuesday,
            wednesday: calendar.wednesday,
            thursday: calendar.thursday,
            friday: calendar.friday,
            saturday: calendar.saturday,
            sunday: calendar.sunday,
            start_date: calendar.start_date.to_string(),
            end_date: calendar.end_date.to_string(),
        });
    }

    let mut gtfs_calendar_dates = vec![];
    if let Some(calender_dates) = gtfs.calendar_dates {
        for calendar_date in calender_dates? {
            gtfs_calendar_dates.push(GtfsCalendarDate {
                service_id: calendar_date.service_id.clone(),
                date: calendar_date.date.to_string(),
                exception_type: match calendar_date.exception_type {
                    gtfs_structures::Exception::Added => GtfsExceptionType::Added,
                    gtfs_structures::Exception::Deleted => GtfsExceptionType::Deleted,
                },
            });
        }
    }

    log::info!("Serializing data.");
    let buffer = rkyv::to_bytes::<rkyv::rancor::Error>(&GtfsData {
        stops: gtfs_stops,
        stop_times: gtfs_stop_times,
        trips: gtfs_trips,
        routes: gtfs_routes,
        agencies: gtfs_agencies,
        calendars: gtfs_calendars,
        calendar_dates: gtfs_calendar_dates,
    })?;
    Ok(buffer)
}
