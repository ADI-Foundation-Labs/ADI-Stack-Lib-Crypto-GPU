use super::*;

use cuda_bindings::GpuError;
use prover_stages::StageTimer;

pub fn create_proof<
    S: SynthesisMode + 'static,
    C: Circuit<Bn256>,
    T: Transcript<Fr>,
    MC: ManagerConfigs,
>(
    assembly: &DefaultAssembly<S>,
    manager: &mut DeviceMemoryManager<Fr, MC>,
    worker: &Worker,
    setup: &AsyncSetup,
    transcript_params: Option<T::InitializationParameters>,
) -> Result<Proof<Bn256, C>, ProvingError> {
    create_proof_cancellable::<S, C, T, MC>(assembly, manager, worker, setup, transcript_params)
        .map(|proof| proof.expect("cancellation is disabled"))
}

/// As [`create_proof`], but returns `None` when proving stopped between rounds because a
/// cancel was requested through [`prover_stages::cancel`].
pub fn create_proof_cancellable<
    S: SynthesisMode + 'static,
    C: Circuit<Bn256>,
    T: Transcript<Fr>,
    MC: ManagerConfigs,
>(
    assembly: &DefaultAssembly<S>,
    manager: &mut DeviceMemoryManager<Fr, MC>,
    worker: &Worker,
    setup: &AsyncSetup,
    transcript_params: Option<T::InitializationParameters>,
) -> Result<Option<Proof<Bn256, C>>, ProvingError> {
    // if S::PRODUCE_SETUP {
    //     compute_assigments_and_permutations(manager, assembly, worker)?;
    // } else {
    //     assign_cs_variables(manager, assembly, worker)?;
    // }

    let mut stages = StageTimer::new();

    if stages.enter("assignments_and_permutations").is_cancelled() {
        return Ok(None);
    }
    compute_assigments_and_permutations(manager, assembly, worker)?;

    let (mut proof, mut transcript, mut constants, input_values) =
        create_initial_variables::<S, C, T, MC>(assembly, transcript_params);

    let mut msm_handles_round1 = vec![];

    if stages.enter("round1").is_cancelled() {
        return Ok(None);
    }
    // dbg!(1);
    round1(
        manager,
        &assembly,
        &worker,
        &mut proof,
        &mut transcript,
        setup,
        &mut msm_handles_round1,
    )
    .expect("Round 1 failed");

    if stages.enter("round15").is_cancelled() {
        return Ok(None);
    }
    // dbg!(1.5);
    round15(
        manager,
        &assembly,
        &worker,
        &mut proof,
        &mut constants,
        &mut transcript,
        setup,
        msm_handles_round1,
    )
    .expect("Round 1.5 failed");

    if stages.enter("round2").is_cancelled() {
        return Ok(None);
    }
    // dbg!(2);
    round2(
        manager,
        &assembly,
        &worker,
        &mut proof,
        &mut constants,
        &mut transcript,
        setup,
        &input_values,
    )
    .expect("Round 2 failed");

    if stages.enter("round3").is_cancelled() {
        return Ok(None);
    }
    // dbg!(3);
    round3(
        manager,
        &assembly,
        &worker,
        &mut proof,
        &mut constants,
        &mut transcript,
        setup,
    )
    .expect("Round 3 failed");

    if stages.enter("round4").is_cancelled() {
        return Ok(None);
    }
    round4::<_, _, S, _>(manager, &mut proof, &mut constants, &mut transcript)
        .expect("Round 4 failed");

    if stages.enter("round5").is_cancelled() {
        return Ok(None);
    }
    round5(manager, &mut proof, &mut constants, &mut transcript).expect("Round 5 failed");

    stages.finish();

    Ok(Some(proof))
}

fn create_initial_variables<
    S: SynthesisMode + 'static,
    C: Circuit<Bn256>,
    T: Transcript<Fr>,
    MC: ManagerConfigs,
>(
    assembly: &DefaultAssembly<S>,
    transcript_params: Option<T::InitializationParameters>,
) -> (Proof<Bn256, C>, T, ProverConstants<Fr>, Vec<Fr>) {
    assert!(S::PRODUCE_WITNESS);
    assert!(assembly.is_finalized);

    let mut proof = Proof::<Bn256, C>::empty();
    let mut constants = ProverConstants::<Fr>::default();
    constants.non_residues = bellman::plonk::better_cs::generator::make_non_residues::<Fr>(3);
    constants.omega = domain_generator::<Fr>(MC::FULL_SLOT_SIZE);

    let mut transcript = if let Some(params) = transcript_params {
        T::new_from_params(params)
    } else {
        T::new()
    };

    let mut input_values = assembly.input_assingments.to_vec();

    proof.n = assembly.n();
    proof.inputs = input_values.clone();

    for inp in input_values.iter() {
        transcript.commit_field_element(inp);
    }

    (proof, transcript, constants, input_values)
}

#[derive(Debug)]
pub enum ProvingError {
    Synthesis(SynthesisError),
    Gpu(GpuError),
}

impl From<GpuError> for ProvingError {
    fn from(err: GpuError) -> Self {
        Self::Gpu(err)
    }
}

impl From<SynthesisError> for ProvingError {
    fn from(err: SynthesisError) -> Self {
        Self::Synthesis(err)
    }
}
