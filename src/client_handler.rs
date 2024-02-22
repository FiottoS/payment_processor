//! # Client Handler
//!
//! El Controlador de Clientes se encarga del manejo de solicitudes y comunicarse con el Modelo de Cliente
//! para realizar las acciones correspondientes a la solicitud recibida.
//! 
//! 
//! [^note]Nota: _Se podrían realizar verificaciones en cada método para asegurarse de que la información enviada_
//! _a través de la solicitud es correcta._


use actix_web::{post, get, web, HttpResponse, Responder};
use crate::client_model::Client;
use std::sync::{Mutex, Arc};

use rust_decimal::Decimal;
use std::collections::HashMap;
// use tokio::sync::RwLock;

/// Se define la estructura AppState, que describe el estado del servicio en base a los clientes que estan siendo
/// registrados, los balances/saldos correspondientes a los mismos y las transacciones que se solicitan realizar.
/// # Arguments
/// * clients
/// * balances
/// * transaction_data
pub struct AppState {
    pub balances: Arc<Mutex<HashMap<u16, Decimal>>>,
    pub transaction_data:Arc<Mutex<HashMap<u16, Decimal>>>,
    pub clients: Arc<Mutex<Vec<Client>>>,
}

/// Recibe la solicitud de crear un nuevo cliente y utiliza recursos del Modelo de Cliente para dar el alta de 
/// cliente, corroborando a su vez que el dni del cliente no se encuentre ya registrado. Finalmente se encarga 
/// de responder la solicitud.
#[post("/new_client")]
pub async fn new_client(client: web::Json<Client>, data: web::Data<AppState>) -> impl Responder {
    
    let mut clients = data.clients.lock().unwrap();

    let mut balances = data.balances.lock().unwrap();

    // Verificar si el document_number ya existe
    if Client::dni_exists(client.document_number().to_owned(),&clients) {
        return HttpResponse::BadRequest().body("El número de documento ya existe");
    }
    // Crear nuevo cliente con ID único
    let new_client = Client::new(
        client.client_name().to_owned(),
        client.birth_date(),
        client.document_number().to_owned(),
        client.country().to_owned(),
        &clients,
    );

    //agrego a balances id usuario nuevo con 0.0 por defecto.
    balances.insert(new_client.id(), Decimal::default());

    // Agregar el nuevo cliente a la lista
    clients.push(new_client.clone());

    // Devolver el ID del nuevo cliente
    HttpResponse::Created().body(format!("ID del nuevo cliente: {}", new_client.id()))
}

/// Recibe la solicitud de acreditar cierto monto a un cliente especificado mediante su ID. En caso de éxito,
/// se actualiza el HashMap **_balances_** y finalmente se encarga de responder la solicitud recibida.
#[post("/new_credit_transaction")]
pub async fn new_credit_transaction(
    transaction_data: web::Json<HashMap<String, serde_json::Value>>,
    data: web::Data<AppState>) -> impl Responder {

    //Captura de errores    
    let e1 = String::from("El valor excede el rango de u16");
    let e2 = String::from("El usuario no existe");
    match Client::credit_transaction(transaction_data, &data) {
        Ok((user_id, actual_balance))=> {
            HttpResponse::Ok().body(format!("Transacción exitosa user_id {}: new_balance {}",
                                                                        user_id, actual_balance))}
        Err(e1) => {HttpResponse::BadRequest().body("El ID o valor ingresado no es válido")}
        Err(e2) => {HttpResponse::BadRequest().body("El usuario no existe")}
        Err(_) => {HttpResponse::InternalServerError().body("El valor ingresado no es válido")}
    }
}

/// Recibe la solicitud de debitar cierto monto a un cliente especificado mediante su ID. En caso de éxito,
/// actualiza el HashMap **_balances_** y finalmente se encarga de responder la solicitud recibida.
#[post("/new_debit_transaction")]
pub async fn new_debit_transaction(
    transaction_data: web::Json<HashMap<String, serde_json::Value>>,
    data: web::Data<AppState>) -> impl Responder {

    //Captura de errores
    let e1 = String::from("El valor excede el rango de u16");
    let e2 = String::from("El usuario no existe");
    match Client::debit_transaction(transaction_data, &data) {
        Ok((user_id, actual_balance))=> {
            HttpResponse::Ok().body(format!("Transacción exitosa user_id {}: new_balance {}",
                                                                         user_id, actual_balance))}
        Err(e1) => {HttpResponse::BadRequest().body("El ID o valor ingresado no es válido")}
        Err(e2) => {HttpResponse::BadRequest().body("El usuario no existe")}
        Err(_) => {HttpResponse::InternalServerError().body("El valor ingresado no es válido")}
    }
}

/// Recibe una solicitud de la información y balance/saldo de un cliente específico, brindando como dato su ID.
/// Busca al cliente y finalmente se encarga de responder la solicitud recibida.
#[get("/client_balance/{user_id}")]
pub async fn client_balance(path: web::Path<u16>,data: web::Data<AppState>) -> impl Responder {

    let client_id = path.into_inner();

    // Obtener el balance correspondiente al cliente
    let balances = data.balances.lock().unwrap();

    let balance = match balances.get(&client_id) {
        Some(b) => b.to_string(),
        None => "Cliente no encontrado".to_string(),
    };

    if balance == "Cliente no encontrado" {
        HttpResponse::BadRequest().body("Cliente no encontrado. El ID ingresado no es válido");
    }

    let show_client = serde_json::to_string(&Client::find_client_by_id(&data.clients,client_id)).unwrap();
    // Retornar el balance del cliente como respuesta
    HttpResponse::Ok().body(format!("Balance del cliente ID {}: {} \nDatos del cliente: {}", client_id, balance, show_client))
}

/// Recibe la solicitud de almacenar el balance actual que se registra en el servicio. Se comunica con el Modelo del Cliente
/// el cual se encarga del almacenamiento en el directorio data, y luego se ponen en cero los balances de todos los clientes.
/// Finalmente brinda un mensaje de datos guardados en caso de éxito.
#[post("/store_balances")]
pub async fn store_balances(data: web::Data<AppState>) -> impl Responder {

if let Err(e) = Client::save_balance(&data) {
    eprintln!("Error al guardar los balances: {}", e);
    HttpResponse::InternalServerError().body(format!("Error al guardar los balances: {}", e));
}
HttpResponse::Ok().body(format!("Datos guardados"))
}