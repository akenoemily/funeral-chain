#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::fmt;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ServiceProvider {
    id: u64,
    name: String,
    service_type: String,
    contact_info: String,
    created_at: u64,
    average_rating: f32,
    reviews: Vec<Review>,
    availability: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Booking {
    id: u64,
    service_provider_id: u64,
    client_id: u64,
    service_date: u64,
    service_type: String,
    status: BookingStatusEnum,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Client {
    id: u64,
    name: String,
    contact_info: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Review {
    client_id: u64,
    rating: u8,
    comment: String,
    created_at: u64,
}

#[derive(
    candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug,
)]
enum BookingStatusEnum {
    #[default]
    Pending,
    Confirmed,
    Canceled,
    Completed,
}

impl fmt::Display for BookingStatusEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            BookingStatusEnum::Pending => "Pending",
            BookingStatusEnum::Confirmed => "Confirmed",
            BookingStatusEnum::Canceled => "Canceled",
            BookingStatusEnum::Completed => "Completed",
        };
        write!(f, "{}", status_str)
    }
}

impl Storable for ServiceProvider {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for ServiceProvider {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Booking {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Booking {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Client {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Client {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static SERVICE_PROVIDER_STORAGE: RefCell<StableBTreeMap<u64, ServiceProvider, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static BOOKING_STORAGE: RefCell<StableBTreeMap<u64, Booking, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static CLIENT_STORAGE: RefCell<StableBTreeMap<u64, Client, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct ServiceProviderPayload {
    name: String,
    service_type: String,
    contact_info: String,
    availability: Vec<u64>,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct BookingPayload {
    service_provider_id: u64,
    client_id: u64,
    service_date: u64,
    service_type: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct ClientPayload {
    name: String,
    contact_info: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct ReviewPayload {
    booking_id: u64,
    rating: u8,
    comment: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Message {
    Success(String),
    Error(String),
    NotFound(String),
    InvalidPayload(String),
}

#[ic_cdk::update]
fn create_service_provider(payload: ServiceProviderPayload) -> Result<ServiceProvider, Message> {
    if payload.name.is_empty() || payload.service_type.is_empty() || payload.contact_info.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'name', 'service_type', and 'contact_info' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let service_provider = ServiceProvider {
        id,
        name: payload.name,
        service_type: payload.service_type,
        contact_info: payload.contact_info,
        created_at: current_time(),
        average_rating: 0.0,
        reviews: Vec::new(),
        availability: payload.availability,
    };
    SERVICE_PROVIDER_STORAGE.with(|storage| storage.borrow_mut().insert(id, service_provider.clone()));
    Ok(service_provider)
}

#[ic_cdk::query]
fn search_service_providers(query: String, filter: Option<String>) -> Result<Vec<ServiceProvider>, Message> {
    SERVICE_PROVIDER_STORAGE.with(|storage| {
        let providers: Vec<ServiceProvider> = storage
            .borrow()
            .iter()
            .map(|(_, provider)| provider.clone())
            .filter(|provider| {
                provider.name.contains(&query)
                    || provider.service_type.contains(&query)
                    || provider.contact_info.contains(&query)
            })
            .filter(|provider| {
                if let Some(ref f) = filter {
                    provider.service_type.contains(f)
                } else {
                    true
                }
            })
            .collect();

        if providers.is_empty() {
            Err(Message::NotFound("No service providers found".to_string()))
        } else {
            Ok(providers)
        }
    })
}

#[ic_cdk::update]
fn create_booking(payload: BookingPayload) -> Result<Booking, Message> {
    if payload.service_date == 0 {
        return Err(Message::InvalidPayload(
            "Invalid service date.".to_string(),
        ));
    }

    let provider = SERVICE_PROVIDER_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&payload.service_provider_id)
            .map(|provider| provider.clone())
    });

    if provider.is_none() {
        return Err(Message::InvalidPayload(
            "Invalid service_provider_id provided.".to_string(),
        ));
    }

    let provider = provider.unwrap();

    // Check if the service date is available
    if !provider.availability.contains(&payload.service_date) {
        return Err(Message::Error("Service provider is not available on the selected date.".to_string()));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let booking = Booking {
        id,
        service_provider_id: payload.service_provider_id,
        client_id: payload.client_id,
        service_date: payload.service_date,
        service_type: payload.service_type,
        status: BookingStatusEnum::Pending,
        created_at: current_time(),
    };
    BOOKING_STORAGE.with(|storage| storage.borrow_mut().insert(id, booking.clone()));
    Ok(booking)
}

#[ic_cdk::update]
fn reschedule_booking(booking_id: u64, new_date: u64) -> Result<Message, Message> {
    let booking = BOOKING_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, booking)| booking.id == booking_id)
            .map(|(_, booking)| booking.clone())
    });

    if booking.is_none() {
        return Err(Message::NotFound("Booking not found".to_string()));
    }

    let mut booking = booking.unwrap();

    if booking.status != BookingStatusEnum::Pending {
        return Err(Message::Error(
            "Only pending bookings can be rescheduled.".to_string(),
        ));
    }

    let provider = SERVICE_PROVIDER_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&booking.service_provider_id)
            .map(|provider| provider.clone())
    });

    if provider.is_none() {
        return Err(Message::NotFound("Service provider not found".to_string()));
    }

    let provider = provider.unwrap();

    // Check if the new date is available
    if !provider.availability.contains(&new_date) {
        return Err(Message::Error("Service provider is not available on the new date.".to_string()));
    }

    booking.service_date = new_date;
    BOOKING_STORAGE.with(|storage| storage.borrow_mut().insert(booking_id, booking));

    Ok(Message::Success("Booking rescheduled.".to_string()))
}

#[ic_cdk::update]
fn create_client(payload: ClientPayload) -> Result<Client, Message> {
    if payload.name.is_empty() || payload.contact_info.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'name' and 'contact_info' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let client = Client {
        id,
        name: payload.name,
        contact_info: payload.contact_info,
    };
    CLIENT_STORAGE.with(|storage| storage.borrow_mut().insert(id, client.clone()));
    Ok(client)
}

#[ic_cdk::query]
fn get_client_bookings(client_id: u64) -> Result<Vec<Booking>, Message> {
    BOOKING_STORAGE.with(|storage| {
        let bookings: Vec<Booking> = storage
            .borrow()
            .iter()
            .filter(|(_, booking)| booking.client_id == client_id)
            .map(|(_, booking)| booking.clone())
            .collect();

        if bookings.is_empty() {
            Err(Message::NotFound("No bookings found for this client.".to_string()))
        } else {
            Ok(bookings)
        }
    })
}

#[ic_cdk::update]
fn add_review(payload: ReviewPayload) -> Result<Message, Message> {
    let booking = BOOKING_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&payload.booking_id)
            .map(|booking| booking.clone())
    });

    if booking.is_none() {
        return Err(Message::NotFound("Booking not found.".to_string()));
    }

    let booking = booking.unwrap();

    if booking.status != BookingStatusEnum::Completed {
        return Err(Message::Error("Only completed bookings can be reviewed.".to_string()));
    }

    let review = Review {
        client_id: booking.client_id,
        rating: payload.rating,
        comment: payload.comment,
        created_at: current_time(),
    };

    SERVICE_PROVIDER_STORAGE.with(|storage| {
        if let Some(mut provider) = storage.borrow_mut().get(&booking.service_provider_id) {
            provider.reviews.push(review);

            // Update the average rating
            let total_ratings: u32 = provider.reviews.iter().map(|r| r.rating as u32).sum();
            provider.average_rating = total_ratings as f32 / provider.reviews.len() as f32;

            storage.borrow_mut().insert(provider.id, provider);
        }
    });

    Ok(Message::Success("Review added.".to_string()))
}

#[ic_cdk::update]
fn confirm_booking(booking_id: u64) -> Result<Message, Message> {
    let booking = BOOKING_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&booking_id)
            .map(|booking| booking.clone())
    });

    if booking.is_none() {
        return Err(Message::NotFound("Booking not found".to_string()));
    }

    let mut booking = booking.unwrap();

    if booking.status == BookingStatusEnum::Confirmed {
        return Err(Message::Error(
            "Booking is already confirmed.".to_string(),
        ));
    }

    booking.status = BookingStatusEnum::Confirmed;
    BOOKING_STORAGE.with(|storage| storage.borrow_mut().insert(booking_id, booking));

    Ok(Message::Success("Booking confirmed.".to_string()))
}

#[ic_cdk::update]
fn cancel_booking(booking_id: u64) -> Result<Message, Message> {
    let booking = BOOKING_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&booking_id)
            .map(|booking| booking.clone())
    });

    if booking.is_none() {
        return Err(Message::NotFound("Booking not found".to_string()));
    }

    let mut booking = booking.unwrap();

    if booking.status == BookingStatusEnum::Canceled {
        return Err(Message::Error(
            "Booking is already canceled.".to_string(),
        ));
    }

    booking.status = BookingStatusEnum::Canceled;
    BOOKING_STORAGE.with(|storage| storage.borrow_mut().insert(booking_id, booking));

    Ok(Message::Success("Booking canceled.".to_string()))
}

#[ic_cdk::query]
fn get_service_provider_history(service_provider_id: u64) -> Result<Vec<Booking>, Message> {
    BOOKING_STORAGE.with(|storage| {
        let bookings: Vec<Booking> = storage
            .borrow()
            .iter()
            .filter(|(_, booking)| booking.service_provider_id == service_provider_id)
            .map(|(_, booking)| booking.clone())
            .collect();

        if bookings.is_empty() {
            Err(Message::NotFound("No bookings found for this service provider.".to_string()))
        } else {
            Ok(bookings)
        }
    })
}

fn current_time() -> u64 {
    time()
}

ic_cdk::export_candid!();
