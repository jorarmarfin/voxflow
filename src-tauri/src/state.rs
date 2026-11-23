//! Máquina de estados de grabación: IDLE → RECORDING → TRANSCRIBING → READY → IDLE.
//! Los errores recuperables vuelven a IDLE. Impide grabaciones simultáneas.
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecState {
    Idle,
    Recording,
    Transcribing,
    Ready,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: RecState,
    pub action: &'static str,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "acción '{}' no válida en estado {:?}",
            self.action, self.from
        )
    }
}

#[derive(Debug)]
pub struct Machine {
    state: RecState,
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            state: RecState::Idle,
        }
    }
}

impl Machine {
    pub fn state(&self) -> RecState {
        self.state
    }

    fn go(
        &mut self,
        from: RecState,
        to: RecState,
        action: &'static str,
    ) -> Result<RecState, InvalidTransition> {
        if self.state == from {
            self.state = to;
            Ok(to)
        } else {
            Err(InvalidTransition {
                from: self.state,
                action,
            })
        }
    }

    pub fn start(&mut self) -> Result<RecState, InvalidTransition> {
        self.go(RecState::Idle, RecState::Recording, "start")
    }
    pub fn stop(&mut self) -> Result<RecState, InvalidTransition> {
        self.go(RecState::Recording, RecState::Transcribing, "stop")
    }
    #[allow(dead_code)] // se usa desde la Fase 2 (SpeechEngine)
    pub fn finish(&mut self) -> Result<RecState, InvalidTransition> {
        self.go(RecState::Transcribing, RecState::Ready, "finish")
    }
    /// Cancela desde cualquier estado activo y libera recursos (vuelve a IDLE).
    pub fn cancel(&mut self) -> RecState {
        self.state = RecState::Idle;
        self.state
    }
    /// Texto entregado o descartado.
    #[allow(dead_code)] // se usa desde la Fase 2
    pub fn dismiss(&mut self) -> Result<RecState, InvalidTransition> {
        self.go(RecState::Ready, RecState::Idle, "dismiss")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flujo_completo() {
        let mut m = Machine::default();
        assert_eq!(m.start(), Ok(RecState::Recording));
        assert_eq!(m.stop(), Ok(RecState::Transcribing));
        assert_eq!(m.finish(), Ok(RecState::Ready));
        assert_eq!(m.dismiss(), Ok(RecState::Idle));
    }

    #[test]
    fn impide_grabaciones_simultaneas() {
        let mut m = Machine::default();
        m.start().unwrap();
        assert!(m.start().is_err());
        assert_eq!(m.state(), RecState::Recording);
    }

    #[test]
    fn cancelar_vuelve_a_idle_desde_cualquier_estado() {
        for setup in 0..4 {
            let mut m = Machine::default();
            if setup > 0 {
                m.start().unwrap();
            }
            if setup > 1 {
                m.stop().unwrap();
            }
            if setup > 2 {
                m.finish().unwrap();
            }
            assert_eq!(m.cancel(), RecState::Idle);
        }
    }

    #[test]
    fn transiciones_invalidas_no_cambian_estado() {
        let mut m = Machine::default();
        assert!(m.stop().is_err());
        assert!(m.finish().is_err());
        assert!(m.dismiss().is_err());
        assert_eq!(m.state(), RecState::Idle);
    }
}
