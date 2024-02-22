use actix_web::{web, App, HttpServer};
use std::sync::{Mutex, Arc};
use crate::client_model::Client;
use std::collections::HashMap;
use rust_decimal::Decimal;

mod client_model;
mod client_handler;

#[actix_web::main]
// #[tokio::main]
async fn main() -> std::io::Result<()> {
    // Crear un vector de clientes envuelto en un Mutex y un Arc
    let clientes: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    // Configurar el servicio de balances
    let balances: Arc<Mutex<HashMap<u16, Decimal>>> = Arc::new(Mutex::new(HashMap::new()));

    let transaction_data: Arc<Mutex<HashMap<u16, Decimal>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Definir el estado de la aplicación
    let app_state = web::Data::new(client_handler::AppState {
        transaction_data: Arc::clone(&transaction_data),
        balances: Arc::clone(&balances),
        clients: Arc::clone(&clientes),
    });

    // Configurar el servidor Actix Web
    
    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/client")
                .app_data(app_state.clone()) // Compartir el estado de la aplicación con los controladores  
                // .route("/new_client", web::post().to(client_controllers::crear_cliente)) // Ruta para crear clientes
                .app_data(balances.clone())
                .service(client_handler::new_client)
                .service(client_handler::new_credit_transaction)
                .service(client_handler::new_debit_transaction)
                .service(client_handler::store_balances)
                .service(client_handler::client_balance)
            )

    })
    .bind("127.0.0.1:8080")? // Enlazar el servidor a la dirección y puerto
    .run() // Iniciar el servidor
    .await // Esperar a que el servidor finalice
}