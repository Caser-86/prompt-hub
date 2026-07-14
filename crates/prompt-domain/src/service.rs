use crate::{Actor, DomainError};

pub(crate) fn require_user_actor(actor: Actor) -> Result<(), DomainError> {
    match actor {
        Actor::User => Ok(()),
        Actor::Ai | Actor::Mcp => Err(DomainError::ExternalWriteToPublishedPrompt),
    }
}
