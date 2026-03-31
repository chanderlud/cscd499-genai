#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::Result;

    #[test]
    fn observes_basic_manual_reset_transitions() -> Result<()> {
        let states = run_manual_reset_event_script(
            false,
            &[
                EventAction::Check,
                EventAction::Set,
                EventAction::Check,
                EventAction::Reset,
                EventAction::Check,
            ],
        )?;

        assert_eq!(states, vec![false, true, true, false, false]);
        Ok(())
    }

    #[test]
    fn initially_signaled_event_stays_signaled_until_reset() -> Result<()> {
        let states = run_manual_reset_event_script(
            true,
            &[
                EventAction::Check,
                EventAction::Check,
                EventAction::Reset,
                EventAction::Check,
                EventAction::Set,
                EventAction::Check,
            ],
        )?;

        assert_eq!(states, vec![true, true, false, false, true, true]);
        Ok(())
    }

    #[test]
    fn empty_script_returns_empty_observations() -> Result<()> {
        let states = run_manual_reset_event_script(true, &[])?;
        assert!(states.is_empty());
        Ok(())
    }
}
