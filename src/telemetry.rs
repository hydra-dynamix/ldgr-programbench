use std::env;
use std::path::PathBuf;

use ldgr::telemetry::buffer::LocalSequenceBuffer;
use ldgr::telemetry::transition::{
    NumericalProtocol, StateCode, CANCELLED, COMPLETED_INCONCLUSIVE, COMPLETED_NEGATIVE,
    COMPLETED_POSITIVE, OPERATIONAL_FAILURE, PENDING, RUNNING,
};

pub(crate) const PROGRAMBENCH_CUSTODY_VERIFIED: StateCode = 8;
pub(crate) const PROGRAMBENCH_REPRODUCTION_PREPARED: StateCode = 9;
pub(crate) const PROGRAMBENCH_ATTEMPTS_RECORDED: StateCode = 10;
pub(crate) const PROGRAMBENCH_EVIDENCE_FINALIZED: StateCode = 11;

const PROGRAMBENCH_REPRODUCTION_STATES: &[StateCode] = &[
    PENDING,
    RUNNING,
    PROGRAMBENCH_CUSTODY_VERIFIED,
    PROGRAMBENCH_REPRODUCTION_PREPARED,
    PROGRAMBENCH_ATTEMPTS_RECORDED,
    PROGRAMBENCH_EVIDENCE_FINALIZED,
    COMPLETED_POSITIVE,
    COMPLETED_NEGATIVE,
    COMPLETED_INCONCLUSIVE,
    OPERATIONAL_FAILURE,
    CANCELLED,
];

const PROGRAMBENCH_REPRODUCTION_TRANSITIONS: &[(StateCode, StateCode)] = &[
    (PENDING, RUNNING),
    (PENDING, OPERATIONAL_FAILURE),
    (PENDING, CANCELLED),
    (RUNNING, PROGRAMBENCH_CUSTODY_VERIFIED),
    (RUNNING, OPERATIONAL_FAILURE),
    (RUNNING, CANCELLED),
    (
        PROGRAMBENCH_CUSTODY_VERIFIED,
        PROGRAMBENCH_REPRODUCTION_PREPARED,
    ),
    (PROGRAMBENCH_CUSTODY_VERIFIED, OPERATIONAL_FAILURE),
    (PROGRAMBENCH_CUSTODY_VERIFIED, CANCELLED),
    (
        PROGRAMBENCH_REPRODUCTION_PREPARED,
        PROGRAMBENCH_ATTEMPTS_RECORDED,
    ),
    (PROGRAMBENCH_REPRODUCTION_PREPARED, OPERATIONAL_FAILURE),
    (PROGRAMBENCH_REPRODUCTION_PREPARED, CANCELLED),
    (
        PROGRAMBENCH_ATTEMPTS_RECORDED,
        PROGRAMBENCH_EVIDENCE_FINALIZED,
    ),
    (PROGRAMBENCH_ATTEMPTS_RECORDED, OPERATIONAL_FAILURE),
    (PROGRAMBENCH_ATTEMPTS_RECORDED, CANCELLED),
    (PROGRAMBENCH_EVIDENCE_FINALIZED, COMPLETED_POSITIVE),
    (PROGRAMBENCH_EVIDENCE_FINALIZED, COMPLETED_NEGATIVE),
    (PROGRAMBENCH_EVIDENCE_FINALIZED, COMPLETED_INCONCLUSIVE),
    (PROGRAMBENCH_EVIDENCE_FINALIZED, OPERATIONAL_FAILURE),
    (PROGRAMBENCH_EVIDENCE_FINALIZED, CANCELLED),
];

pub(crate) const PROGRAMBENCH_REPRODUCTION_V1: NumericalProtocol = NumericalProtocol::new(
    "/sequences/programbench-reproduction/v1",
    PENDING,
    PROGRAMBENCH_REPRODUCTION_STATES,
    PROGRAMBENCH_REPRODUCTION_TRANSITIONS,
    32,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramBenchReproductionStep {
    InputsPrepared,
    ReproductionPrepared,
    AttemptsRecorded,
    EvidenceFinalized,
}

impl ProgramBenchReproductionStep {
    const fn state_code(self) -> StateCode {
        match self {
            Self::InputsPrepared => PROGRAMBENCH_CUSTODY_VERIFIED,
            Self::ReproductionPrepared => PROGRAMBENCH_REPRODUCTION_PREPARED,
            Self::AttemptsRecorded => PROGRAMBENCH_ATTEMPTS_RECORDED,
            Self::EvidenceFinalized => PROGRAMBENCH_EVIDENCE_FINALIZED,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramBenchReproductionTerminal {
    CompletedPositive,
    CompletedNegative,
    CompletedInconclusive,
    OperationalFailure,
    Cancelled,
}

impl ProgramBenchReproductionTerminal {
    const fn state_code(self) -> StateCode {
        match self {
            Self::CompletedPositive => COMPLETED_POSITIVE,
            Self::CompletedNegative => COMPLETED_NEGATIVE,
            Self::CompletedInconclusive => COMPLETED_INCONCLUSIVE,
            Self::OperationalFailure => OPERATIONAL_FAILURE,
            Self::Cancelled => CANCELLED,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ProgramBenchReproductionTelemetry {
    buffer: Option<LocalSequenceBuffer<'static>>,
}

impl ProgramBenchReproductionTelemetry {
    pub(crate) fn begin_running() -> Self {
        let buffer = telemetry_ldgr_home().and_then(Self::begin_buffer_at);
        let mut telemetry = Self { buffer };
        telemetry.submit(RUNNING);
        telemetry
    }

    pub(crate) fn record_step(&mut self, step: ProgramBenchReproductionStep) {
        self.submit(step.state_code());
    }

    pub(crate) fn finish(&mut self, terminal: ProgramBenchReproductionTerminal) {
        self.submit(terminal.state_code());
    }

    fn begin_buffer_at(ldgr_home: PathBuf) -> Option<LocalSequenceBuffer<'static>> {
        LocalSequenceBuffer::begin_after_commit(ldgr_home, &PROGRAMBENCH_REPRODUCTION_V1)
            .ok()
            .flatten()
    }

    #[cfg(test)]
    fn begin_running_at(ldgr_home: impl Into<PathBuf>) -> Self {
        let mut telemetry = Self {
            buffer: Self::begin_buffer_at(ldgr_home.into()),
        };
        telemetry.submit(RUNNING);
        telemetry
    }

    fn submit(&mut self, state: StateCode) {
        let Some(buffer) = self.buffer.as_mut() else {
            return;
        };
        if buffer.submit_committed(state).is_err() {
            self.buffer = None;
        }
    }
}

fn telemetry_ldgr_home() -> Option<PathBuf> {
    env::var_os("LDGR_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".ldgr"))
        })
        .or_else(|| {
            env::var_os("USERPROFILE")
                .map(PathBuf::from)
                .map(|home| home.join(".ldgr"))
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use ldgr::telemetry::adapter_conformance::{
        verify_adapter_telemetry_conformance, TerminalPath,
    };
    use ldgr::telemetry::transition::NormalizedTerminal;
    use ldgr::telemetry::{
        save_telemetry_consent, TelemetryConsent, TelemetryConsentDecision,
        TELEMETRY_PENDING_DIRECTORY,
    };

    use super::*;

    const POSITIVE_PATH: &[StateCode] = &[
        PENDING,
        RUNNING,
        PROGRAMBENCH_CUSTODY_VERIFIED,
        PROGRAMBENCH_REPRODUCTION_PREPARED,
        PROGRAMBENCH_ATTEMPTS_RECORDED,
        PROGRAMBENCH_EVIDENCE_FINALIZED,
        COMPLETED_POSITIVE,
    ];
    const NEGATIVE_PATH: &[StateCode] = &[
        PENDING,
        RUNNING,
        PROGRAMBENCH_CUSTODY_VERIFIED,
        PROGRAMBENCH_REPRODUCTION_PREPARED,
        PROGRAMBENCH_ATTEMPTS_RECORDED,
        PROGRAMBENCH_EVIDENCE_FINALIZED,
        COMPLETED_NEGATIVE,
    ];
    const INCONCLUSIVE_PATH: &[StateCode] = &[
        PENDING,
        RUNNING,
        PROGRAMBENCH_CUSTODY_VERIFIED,
        PROGRAMBENCH_REPRODUCTION_PREPARED,
        PROGRAMBENCH_ATTEMPTS_RECORDED,
        PROGRAMBENCH_EVIDENCE_FINALIZED,
        COMPLETED_INCONCLUSIVE,
    ];
    const OPERATIONAL_FAILURE_PATH: &[StateCode] = &[
        PENDING,
        RUNNING,
        PROGRAMBENCH_CUSTODY_VERIFIED,
        OPERATIONAL_FAILURE,
    ];
    const CANCELLED_PATH: &[StateCode] = &[PENDING, RUNNING, CANCELLED];

    fn enable(ldgr_home: &Path) -> Result<(), Box<dyn std::error::Error>> {
        save_telemetry_consent(
            ldgr_home,
            &TelemetryConsent::current(TelemetryConsentDecision::Enabled),
        )?;
        Ok(())
    }

    fn pending_payloads(ldgr_home: &Path) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let route = ldgr_home
            .join(TELEMETRY_PENDING_DIRECTORY)
            .join("programbench-reproduction/v1");
        if !route.exists() {
            return Ok(Vec::new());
        }
        let mut payloads = Vec::new();
        for entry in fs::read_dir(route)? {
            payloads.push(fs::read(entry?.path())?);
        }
        payloads.sort();
        Ok(payloads)
    }

    #[test]
    fn programbench_reproduction_protocol_conforms_to_core_adapter_contract(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let report = verify_adapter_telemetry_conformance(
            &PROGRAMBENCH_REPRODUCTION_V1,
            &[
                TerminalPath::new(NormalizedTerminal::CompletedPositive, POSITIVE_PATH),
                TerminalPath::new(NormalizedTerminal::CompletedNegative, NEGATIVE_PATH),
                TerminalPath::new(NormalizedTerminal::CompletedInconclusive, INCONCLUSIVE_PATH),
                TerminalPath::new(
                    NormalizedTerminal::OperationalFailure,
                    OPERATIONAL_FAILURE_PATH,
                ),
                TerminalPath::new(NormalizedTerminal::Cancelled, CANCELLED_PATH),
            ],
        )?;
        assert_eq!(report.endpoint, "/sequences/programbench-reproduction/v1");
        assert!(report.terminal_payloads.iter().any(|payload| {
            payload.terminal == NormalizedTerminal::CompletedNegative
                && payload.payload == b"[0,1,8,9,10,11,4]"
        }));
        Ok(())
    }

    #[test]
    fn captured_programbench_payload_is_bare_numeric_state_without_benchmark_content(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let home = tempfile::tempdir()?;
        enable(home.path())?;

        let mut telemetry = ProgramBenchReproductionTelemetry::begin_running_at(home.path());
        telemetry.record_step(ProgramBenchReproductionStep::InputsPrepared);
        telemetry.record_step(ProgramBenchReproductionStep::ReproductionPrepared);
        telemetry.record_step(ProgramBenchReproductionStep::AttemptsRecorded);
        telemetry.record_step(ProgramBenchReproductionStep::EvidenceFinalized);
        telemetry.finish(ProgramBenchReproductionTerminal::CompletedNegative);

        let payloads = pending_payloads(home.path())?;
        assert_eq!(payloads, vec![b"[0,1,8,9,10,11,4]".to_vec()]);
        let text = std::str::from_utf8(&payloads[0])?;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(text)?,
            serde_json::json!([0, 1, 8, 9, 10, 11, 4])
        );
        for prohibited in [
            "sharkdp",
            "hyperfine",
            "wfxr",
            "code-minimap",
            "google",
            "brotli",
            "yaa110",
            "nomino",
            "submission",
            "eval",
            "stdout",
            "stderr",
            "target",
            "answer",
            "command",
            "archive",
            "benchmarks",
            "artifact",
        ] {
            assert!(!text.contains(prohibited), "payload leaked `{prohibited}`");
        }
        Ok(())
    }
}
