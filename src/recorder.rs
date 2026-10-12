//! Tracks whether a recording is in progress, so the server can reject
//! out-of-order `record` and `transcribe` requests.

use anyhow::{Context, Result, bail};

#[derive(Debug)]
pub struct Recorder<R> {
    active: Option<R>,
}

impl<R> Default for Recorder<R> {
    fn default() -> Self {
        Self { active: None }
    }
}

impl<R> Recorder<R> {
    /// Begins a recording by calling `begin`. Fails if one is already running,
    /// or if `begin` fails; in both cases the state is left unchanged.
    pub fn start(&mut self, begin: impl FnOnce() -> Result<R>) -> Result<()> {
        if self.active.is_some() {
            bail!("already recording");
        }
        self.active = Some(begin()?);
        Ok(())
    }

    /// Ends the current recording and hands it back. Fails if none is running.
    pub fn stop(&mut self) -> Result<R> {
        self.active.take().context("not recording")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;

    use super::Recorder;

    #[test]
    fn stop_without_start_fails() {
        let mut recorder: Recorder<u32> = Recorder::default();
        assert!(recorder.stop().is_err());
    }

    #[test]
    fn start_then_stop_returns_the_recording() {
        let mut recorder = Recorder::default();
        recorder.start(|| Ok(7)).unwrap();
        assert_eq!(recorder.stop().unwrap(), 7);
    }

    #[test]
    fn stop_ends_the_recording() {
        let mut recorder = Recorder::default();
        recorder.start(|| Ok(7)).unwrap();
        recorder.stop().unwrap();
        assert!(recorder.stop().is_err());
    }

    #[test]
    fn second_start_fails_and_keeps_the_first_recording() {
        let mut recorder = Recorder::default();
        recorder.start(|| Ok(1)).unwrap();
        let mut begun = false;
        let err = recorder
            .start(|| {
                begun = true;
                Ok(2)
            })
            .unwrap_err();
        assert!(!begun, "begin must not run while already recording");
        assert_eq!(err.to_string(), "already recording");
        assert_eq!(recorder.stop().unwrap(), 1);
    }

    #[test]
    fn failed_begin_leaves_recorder_idle() {
        let mut recorder: Recorder<u32> = Recorder::default();
        assert!(recorder.start(|| bail!("no microphone")).is_err());
        assert!(recorder.stop().is_err());
    }

    #[test]
    fn can_record_again_after_stop() {
        let mut recorder = Recorder::default();
        recorder.start(|| Ok(1)).unwrap();
        recorder.stop().unwrap();
        recorder.start(|| Ok(2)).unwrap();
        assert_eq!(recorder.stop().unwrap(), 2);
    }
}
