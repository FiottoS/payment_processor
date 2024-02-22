//! # Client Model
//!
//! El Modelo de Cliente se encarga de administrar la información de los datos de clientes.
//! Posee métodos que permiten el alta de nuevos clientes, las transacciones correspondientes
//! a los mismos y la persistencia de los datos.
//! 
//! [^note]Nota: _Podría generarse un trait o incluso un nuevo modelo "Transactions" que se encargue_
//! _exclusivamente de todos los tipos de transacciones y operaciones que podrían requerirse del servicio,_ 
//! _dejando a "Client" exclusivo para gestionar la información del cliente._


use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use std::sync::atomic::{AtomicU16, Ordering};
use crate::client_handler::AppState;
use actix_web::web;

use std::collections::HashMap;
use std::sync::{Mutex, Arc};

use std::fs::{File, read_dir};
use std::io::prelude::*;
use std::path::Path;
use chrono::prelude::*;

static NEXT_ID: AtomicU16 = AtomicU16::new(1);

/// Representa al actor "Cliente" y expone los atributos que le pertenecen.
///
/// # Arguments

/// * id - Option <u16>
/// * client_name - String
/// * birth_date - NaiveDate
/// * document_number - String
/// * country - String
#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Client {

    id: Option<u16>,
    client_name: String,
    birth_date: NaiveDate,
    document_number: String,
    country: String,
}

/// # Client
///
/// Implementación de Client. Su objetivo es administrar la información de nuevos clientes,
/// encargarse de transacciones y de la persistencia de los datos.
/// Para más detalles de los métodos se recomienda explorar la descripción particular de cada uno.
///
impl Client {

/// # Constructor Client
/// Crea una nueva instancia de Client y a su vez le asigna un id único.
/// # Arguments
/// * client_name - 
/// * birth_date - 
/// * document_number - 
/// * country - 
/// * clients - Referencia al grupo de clientes que fue asignado

    pub fn new (client_name: String, birth_date: NaiveDate, 
        document_number: String, country: String, clients: &[Client]) -> Self {

        let id = Self::generate_unique_id(clients); // Generar un ID único
        Client {id:Some(id) , client_name, birth_date, document_number, country }
    }

    /// Este método se encarga de verificar si el dni ya existe o no en la lista de clientes.
    pub fn dni_exists (document_number: String,
        clients: &std::sync::MutexGuard<'_, Vec<Client>> )-> bool {

        if clients.to_owned().iter().any(|c| c.document_number() == document_number) {
            return true;
        }   
        else { return false }
    }
    /// Este método se encarga de generar un ID único, utilizando el static NEXT_ID, el cual siempre incrementa en uno
    /// mientras el servicio se esté ejecutando. En caso de reiniciarse el servicio, también lo hace NEXT_ID.
    fn generate_unique_id(clients: &[Client]) -> u16 {

        let mut new_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        while Self::id_exists(new_id, clients) {
            new_id = Self::generate_unique_id(clients);
        }
        new_id
    }
    /// Este método se encarga de verificar si el ID ya existe o no en la lista de clientes.
    fn id_exists (id: u16, clients: &[Client]) -> bool {
        for client in clients {
            if client.id() == id {
                return true; // Si el ID ya existe en algún cliente, retorna true
            }
        }
        false // Si no se encontró el ID en ningún cliente, retorna false
    }
    /// Este método se encarga de encontrar un cliente en la lista de clientes a través de un ID proporcionado.
    /// # Return 
    /// Option: Si el ID es encontrado, se devuelve el cliente al que le pertenece el ID. Caso contrario retorna _None_.
    pub fn find_client_by_id(clients: &Arc<Mutex<Vec<Client>>>, id_to_find:u16) -> Option<Client> {
        // Bloqueamos el Mutex para acceder al vector de clientes
        let clients_guard = clients.lock().unwrap();
        
        // Utilizamos match para buscar el cliente por su ID
        match clients_guard.iter().find(|client| client.id() == id_to_find) {
            Some(located_client) => Some(located_client.clone()), // Devolvemos el cliente si se encuentra
            None => None, // Si no se encuentra el cliente, devolvemos None
        }
    }
    /// Este método se encarga de obtener el ID y el monto proporcionados y realizar una acreditación de saldo al 
    /// cliente correspondiente.
    pub fn credit_transaction(transaction_data: web::Json<HashMap<String, serde_json::Value>>,
        data:&web::Data<AppState>) -> Result<(u16, Decimal), String> {

        let mut balances = data.balances.lock().unwrap();
        let user_id: u16 = match transaction_data.get("client_id") {
            Some(value) => match value.as_u64() {
                Some(value_u64) => {
                    if value_u64 <= u16::MAX as u64 {
                        value_u64 as u16
                    } else {
                        return Err("El valor excede el rango de u16".into())
                    }
                }
                None => return Err ("Error interno".into()),//panic!("No se pudo convertir user_id a u64"),
            },
            None => return Err ("Error interno".into()),//panic!("No se encontró el valor user_id"),
        };

        let amount: Decimal = match transaction_data.get("credit_amount") {
            Some(value) => match value.as_f64() {
                Some(value_f64) => Decimal::from_f64_retain(value_f64).unwrap(),
                None => return Err ("Error interno".into()), //panic!("No se pudo convertir monto a f64"),
            },
            None => return Err ("Error interno".into()),//panic!("No se encontró el valor monto"),
        };

        let Some(current_balance) = balances.get_mut(&user_id) else {
            return Err ("El usuario no existe".into())//panic!("No existe el usuario!")
        };

        *current_balance = *current_balance + amount;

        return Ok((user_id, *current_balance))
    }
    /// Este método se encarga de obtener el ID y el monto proporcionados y realizar la debitación de saldo al 
    /// cliente correspondiente.
    pub fn debit_transaction(transaction_data: web::Json<HashMap<String, serde_json::Value>>,
        data:&web::Data<AppState>) -> Result<(u16, Decimal), String> {

        let mut balances = data.balances.lock().unwrap();
        let user_id: u16 = match transaction_data.get("client_id") {
            Some(value) => match value.as_u64() {
                Some(value_u64) => {
                    if value_u64 <= u16::MAX as u64 {
                        value_u64 as u16
                    } else {
                        return Err("El valor excede el rango de u16".into()) // panic!("El valor excede el rango de u16");
                    }
                }
                None => return Err ("Error interno".into()),//panic!("No se pudo convertir user_id a u64"),
            },
            None => return Err ("Error interno".into()),//panic!("No se encontró el valor user_id"),
        };

        let amount: Decimal = match transaction_data.get("credit_amount") {
            Some(value) => match value.as_f64() {
                Some(value_f64) => Decimal::from_f64_retain(value_f64).unwrap(),
                None => return Err ("Error interno".into()),//panic!("No se pudo convertir monto a f64"),
            },
            None => return Err ("Error interno".into()),//panic!("No se encontró el valor monto"),
        };

        let Some(current_balance) = balances.get_mut(&user_id) else {
            return Err ("El usuario no existe".into())//panic!("No existe el usuario!")
        };

        *current_balance = *current_balance - amount;

        return Ok((user_id, *current_balance))
    }
    /// Este método se encarga de almacenar los balances de los clientes incorporados desde el inicio del servicio
    /// en un archivo de extensión .DAT con formato *"DDMMYYYY_FILE COUNTER.DAT"*, en el directorio *"data"*,
    /// verificando que el valor correspondiente al contador de archivo no se repita.
    pub fn save_balance(data:&web::Data<AppState>) -> std::io::Result<()> {
        // Obtener el contador del último archivo guardado
        let last_counter_value = Self::last_counter()?;
        
        // Incrementar el contador para obtener el nuevo número
        let new_counter = last_counter_value + 1;
        
        // Obtener la fecha actual
        let date = Local::now();
        let date_str = date.format("%d%m%Y").to_string();
        
        // Construir el nombre del archivo con el nuevo contador
        let file_name = format!("data/{}_{}.DAT", date_str, new_counter);
        
        // Crear el path del archivo
        let path = Path::new(&file_name);
        
        // Crear el archivo
        let mut file = File::create(path)?;
        
        // Escribir los datos en el 
        let mut balances = data.balances.lock().unwrap();
        
        for (id_cliente, balance) in balances.iter() {
            writeln!(file, "{} {}", id_cliente, balance)?;
        };
        println!("Datos guardados en el archivo: {}", file_name);

        for (_, balance) in balances.iter_mut() {
            *balance  = Decimal::default();
        }
        println!("Datos de saldo borrados");
        
        Ok(())
        }
        
    /// Este método se encarga de obtener el valor del contador del último archivo guardado.
    fn last_counter() -> std::io::Result<u32> {
        // Directorio donde se encuentran los archivos .DAT
        let dir = read_dir("./data")?;
        
        // Obtener el contador máximo entre los archivos .DAT
        let mut max_counter = 0;
        for entry in dir {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "DAT" {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if let Some(split) = file_name.split("_").last() {
                                if let Ok(counter) = split.replace(".DAT", "").parse::<u32>() {
                                    if counter > max_counter {
                                        max_counter = counter;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(max_counter)
        }

    pub fn id(&self) -> u16 {
        self.id.unwrap()
    }

    pub fn client_name(&self) -> &str {
        &self.client_name
    }

    pub fn birth_date(&self) -> NaiveDate {
        self.birth_date
    }

    pub fn document_number(&self) -> &str {
        &self.document_number
    }

    pub fn country(&self) -> &str {
        &self.country
    }
}
