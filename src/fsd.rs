#[derive(Debug)]
pub struct LoginRequest {
    pub callsign: String,
}

impl LoginRequest {
    pub fn to_response(&self) -> LoginResponse {
        LoginResponse { callsign: self.callsign.clone() }
    }
}

#[derive(Debug)]
pub struct LoginResponse {
    pub callsign: String,
}

#[derive(Debug)]
pub enum FSDMessage {
    LoginRequest(LoginRequest),
    LoginResponse(LoginResponse),
}

impl FSDMessage {
    pub fn from_string(input: String) -> Result<FSDMessage, String> {
        let fields: Vec<&str> = input.split(':').collect();

        let first_field = fields.get(0).ok_or("Invalid message format")?;
        if first_field.len() < 3 {
            return Err(format!("Invalid message type {}", first_field));
        }

        let msg_type = &first_field[0..3];
        let callsign = first_field[3..].to_string();

        println!("{}", msg_type);
        println!("{}", callsign);

        match msg_type {
            "#AA" => Ok(FSDMessage::LoginRequest(LoginRequest { callsign })),
            _ => Err("Unknown message type".to_string()),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let response = match self {
            FSDMessage::LoginRequest(_) => "".to_string(),
            FSDMessage::LoginResponse(login_response) =>
                format!("#TMSERVER:{}:Welcome to the proxy, we've got fun and games\n", login_response.callsign)
        };
        response.into_bytes()
    }
}