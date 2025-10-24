/// Context passed to all handler methods containing session information
#[derive(Debug, Clone)]
pub struct Context {
    /// Unique session identifier
    pub session_id: String,
    
    /// Request ID for this specific call
    pub request_id: usize,
    
    /// Optional user/auth information
    pub user_id: Option<String>,
    
    /// Timestamp when session was created
    pub created_at: u64,
}

impl Context {
    pub fn new(session_id: String, request_id: usize) -> Self {
        Self {
            session_id,
            request_id,
            user_id: None,
            created_at: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}
