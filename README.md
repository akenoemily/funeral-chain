# FuneralChain

FuneralChain is a decentralized platform built to manage service providers and clients for funeral-related services using the Internet Computer. It allows clients to book services, manage providers, leave reviews, and track the status of their bookings efficiently.

## Features

- **Service Provider Management**: Create, store, and manage funeral service providers.
- **Booking System**: Clients can book services, reschedule, confirm, or cancel bookings.
- **Client Management**: Store client information and manage client interactions with service providers.
- **Review System**: Clients can leave reviews for completed services, and service providers' average ratings are updated accordingly.

## 📋 Requirements

- rustc 1.64 or higher

```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```

- rust wasm32-unknown-unknown target

```bash
$ rustup target add wasm32-unknown-unknown
```

- candid-extractor

```bash
$ cargo install candid-extractor
```

- install `dfx`

```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd icp_rust_boilerplate/
$ dfx help
$ dfx canister --help
```

## 🔄 Update dependencies

Update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:

```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## 🔧 did autogenerate

Add this script to the root directory of the project:

```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:

```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this, run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add a package.json with this content:

```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## 🧪 Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```

### Usage

#### Service Provider

- **Create a Service Provider**: Allows service providers to register with name, contact information, service type, and availability.
- **Search for Service Providers**: Clients can search for providers based on various filters.

#### Client

- **Create a Client**: Register a client with their name and contact details.
- **View Bookings**: Fetch all bookings made by a specific client.

#### Booking

- **Create a Booking**: Clients can create bookings with a selected service provider and a specified service date.
- **Reschedule Booking**: Clients can reschedule bookings if the service provider has availability on a new date.
- **Confirm or Cancel Booking**: Booking statuses can be updated to `Confirmed`, `Canceled`, or `Completed`.

#### Reviews

- **Add Review**: Clients can leave reviews after a service is completed. Reviews update the service provider's average rating.

### Data Structures

#### ServiceProvider
```rust
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
```

#### Client
```rust
struct Client {
    id: u64,
    name: String,
    contact_info: String,
}
```

#### Booking
```rust
struct Booking {
    id: u64,
    service_provider_id: u64,
    client_id: u64,
    service_date: u64,
    service_type: String,
    status: BookingStatusEnum,
    created_at: u64,
}
```

#### Review
```rust
struct Review {
    client_id: u64,
    rating: u8,
    comment: String,
    created_at: u64,
}
```

### API Methods

- `create_service_provider(payload: ServiceProviderPayload) -> Result<ServiceProvider, Message>`
- `search_service_providers(query: String, filter: Option<String>) -> Result<Vec<ServiceProvider>, Message>`
- `create_booking(payload: BookingPayload) -> Result<Booking, Message>`
- `reschedule_booking(booking_id: u64, new_date: u64) -> Result<Message, Message>`
- `create_client(payload: ClientPayload) -> Result<Client, Message>`
- `get_client_bookings(client_id: u64) -> Result<Vec<Booking>, Message>`
- `add_review(payload: ReviewPayload) -> Result<Message, Message>`
- `confirm_booking(booking_id: u64) -> Result<Message, Message>`

### Contributions

Feel free to contribute to this project by opening issues, making pull requests, or suggesting features.