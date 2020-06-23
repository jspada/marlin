/********************************************************************************************

This source file implements prover's zk-proof primitive.

*********************************************************************************************/

use rand_core::OsRng;
use oracle::rndoracle::{ProofError};
use algebra::{Field, PairingEngine, Zero, One};
use ff_fft::{DensePolynomial, DenseOrSparsePolynomial, EvaluationDomain};
use crate::plonk_sponge::FrSponge;
use oracle::sponge::FqSponge;
pub use super::index::Index;
use oracle::utils::Utils;

#[derive(Clone)]
pub struct ProverProof<E: PairingEngine>
{
    // polynomial commitments
    pub l_comm: E::G1Affine,
    pub r_comm: E::G1Affine,
    pub o_comm: E::G1Affine,
    pub z_comm: E::G1Affine,
    pub tlow_comm: E::G1Affine,
    pub tmid_comm: E::G1Affine,
    pub thgh_comm: E::G1Affine,

    // batched commitment opening proofs
    pub proof1: E::G1Affine,
    pub proof2: E::G1Affine,

    // polynomial evaluations
    pub evals : ProofEvaluations<E::Fr>,

    // public part of the witness
    pub public: Vec<E::Fr>
}

impl<E: PairingEngine> ProverProof<E>
{
    // This function constructs prover's zk-proof from the witness & the Index against URS instance
    //     witness: computation witness
    //     index: Index
    //     RETURN: prover's zk-proof
    pub fn create
        <EFqSponge: FqSponge<E::Fq, E::G1Affine, E::Fr>,
         EFrSponge: FrSponge<E::Fr>,
        >
    (
        witness: &Vec::<E::Fr>,
        index: &Index<E>
    ) -> Result<Self, ProofError>
    {
        let n = index.cs.domain.d1.size();
        if witness.len() != 3*n {return Err(ProofError::WitnessCsInconsistent)}

        let mut oracles = RandomOracles::<E::Fr>::zero();
        let mut evals = ProofEvaluations::<E::Fr>::zero();

        // the transcript of the random oracle non-interactive argument
        let mut fq_sponge = EFqSponge::new(index.fq_sponge_params.clone());

        // compute public input polynomial
        let public = witness[0..index.cs.public].to_vec();
        let p = -DensePolynomial::evals_from_coeffs(public.clone(), index.cs.domain.d1).interpolate();

        // compute witness polynomials
        let l = &DensePolynomial::evals_from_coeffs(index.cs.gates.iter().map(|gate| witness[gate.l.0]).collect(), index.cs.domain.d1).interpolate()
            + &DensePolynomial::rand(1, &mut OsRng).mul_by_vanishing_poly(index.cs.domain.d1);
        let r = &DensePolynomial::evals_from_coeffs(index.cs.gates.iter().map(|gate| witness[gate.r.0]).collect(), index.cs.domain.d1).interpolate()
            + &DensePolynomial::rand(1, &mut OsRng).mul_by_vanishing_poly(index.cs.domain.d1);
        let o = &DensePolynomial::evals_from_coeffs(index.cs.gates.iter().map(|gate| witness[gate.o.0]).collect(), index.cs.domain.d1).interpolate()
            + &DensePolynomial::rand(1, &mut OsRng).mul_by_vanishing_poly(index.cs.domain.d1);

        // commit to the l, r, o wire values
        let l_comm = index.urs.get_ref().commit(&l)?;
        let r_comm = index.urs.get_ref().commit(&r)?;
        let o_comm = index.urs.get_ref().commit(&o)?;

        // absorb the public input, l, r, o polycommitments into the argument
        fq_sponge.absorb_fr(&public);
        fq_sponge.absorb_g(&[l_comm, r_comm, o_comm]);

        // sample beta, gamma oracles
        oracles.beta = fq_sponge.challenge();
        oracles.gamma = fq_sponge.challenge();

        // compute permutation polynomial

        let mut z = vec![E::Fr::one(); n+1];
        z.iter_mut().skip(1).enumerate().for_each
        (
            |(j, x)| *x =
                (witness[j] + &(index.cs.sigmal[0][j] * &oracles.beta) + &oracles.gamma) *&
                (witness[j+n] + &(index.cs.sigmal[1][j] * &oracles.beta) + &oracles.gamma) *&
                (witness[j+2*n] + &(index.cs.sigmal[2][j] * &oracles.beta) + &oracles.gamma)
        );
        
        algebra::fields::batch_inversion::<E::Fr>(&mut z[1..=n]);

        (0..n).for_each
        (
            |j|
            {
                let x = z[j];
                z[j+1] *=
                    &(x * &(witness[j] + &(index.cs.sid[j] * &oracles.beta) + &oracles.gamma) *&
                    (witness[j+n] + &(index.cs.sid[j] * &oracles.beta * &index.cs.r) + &oracles.gamma) *&
                    (witness[j+2*n] + &(index.cs.sid[j] * &oracles.beta * &index.cs.o) + &oracles.gamma))
            }
        );

        if z.pop().unwrap() != E::Fr::one() {return Err(ProofError::ProofCreation)};
        let z = DensePolynomial::evals_from_coeffs(z, index.cs.domain.d1).interpolate();

        // commit to z
        let z_comm = index.urs.get_ref().commit(&z)?;

        // absorb the z commitment into the argument and query alpha
        fq_sponge.absorb_g(&[z_comm]);
        oracles.alpha = fq_sponge.challenge();
        let alpsq = oracles.alpha.square();

        // compute quotient polynomial

        // generic constraints contribution
        let t1 =
            &(&(&DensePolynomial::multiply(&[&l, &r, &index.cs.qm], index.cs.domain.d3).interpolate() +
            &(
                &(&DensePolynomial::multiply(&[&l, &index.cs.ql], index.cs.domain.d2) +
                &DensePolynomial::multiply(&[&r, &index.cs.qr], index.cs.domain.d2)) +
                &DensePolynomial::multiply(&[&o, &index.cs.qo], index.cs.domain.d2)
            ).interpolate()) +
            &index.cs.qc) + &p;

        // permutation check contribution
        let t2 = DensePolynomial::multiply
            (&[
                &(&l + &DensePolynomial::from_coefficients_slice(&[oracles.gamma, oracles.beta])),
                &(&r + &DensePolynomial::from_coefficients_slice(&[oracles.gamma, oracles.beta*&index.cs.r])),
                &(&o + &DensePolynomial::from_coefficients_slice(&[oracles.gamma, oracles.beta*&index.cs.o])),
                &z
            ], index.cs.domain.d4);

        let t3 = DensePolynomial::multiply
            (&[
                &(&(&l + &DensePolynomial::from_coefficients_slice(&[oracles.gamma])) + &index.cs.sigmam[0].scale(oracles.beta)),
                &(&(&r + &DensePolynomial::from_coefficients_slice(&[oracles.gamma])) + &index.cs.sigmam[1].scale(oracles.beta)),
                &(&(&o + &DensePolynomial::from_coefficients_slice(&[oracles.gamma])) + &index.cs.sigmam[2].scale(oracles.beta)),
                &index.cs.shift(&z)
            ], index.cs.domain.d4);

        // premutation boundary condition check contribution
        let (t4, res) =
            DenseOrSparsePolynomial::divide_with_q_and_r(&(&z - &DensePolynomial::from_coefficients_slice(&[E::Fr::one()])).into(),
                &DensePolynomial::from_coefficients_slice(&[-E::Fr::one(), E::Fr::one()]).into()).
                map_or(Err(ProofError::PolyDivision), |s| Ok(s))?;
        if res.is_zero() == false {return Err(ProofError::PolyDivision)}

        let (mut t, res) = (&t1 + &(&t2 - &t3).interpolate().scale(oracles.alpha)).
            divide_by_vanishing_poly(index.cs.domain.d1).map_or(Err(ProofError::PolyDivision), |s| Ok(s))?;
        if res.is_zero() == false {return Err(ProofError::PolyDivision)}
        t += &t4.scale(alpsq);

        // split t to fit to the commitment
        let tlow: DensePolynomial<E::Fr>;
        let mut tmid = DensePolynomial::from_coefficients_slice(&[E::Fr::zero()]);
        let mut thgh = DensePolynomial::from_coefficients_slice(&[E::Fr::zero()]);
        if t.coeffs.len() <= n {tlow = t}
        else if t.coeffs.len() <= 2*n
        {
            tlow = DensePolynomial::from_coefficients_slice(&t.coeffs[0..n]);
            tmid = DensePolynomial::from_coefficients_slice(&t.coeffs[n..t.coeffs.len()]);
        }
        else
        {
            tlow = DensePolynomial::from_coefficients_slice(&t.coeffs[0..n]);
            tmid = DensePolynomial::from_coefficients_slice(&t.coeffs[n..2*n]);
            thgh = DensePolynomial::from_coefficients_slice(&t.coeffs[2*n..]);
        }

        // commit to tlow, tmid, thgh
        let tlow_comm = index.urs.get_ref().commit(&tlow)?;
        let tmid_comm = index.urs.get_ref().commit(&tmid)?;
        let thgh_comm = index.urs.get_ref().commit(&thgh)?;

        // absorb the polycommitments into the argument and sample zeta
        
        fq_sponge.absorb_g(&[tlow_comm, tmid_comm, thgh_comm]);
        oracles.zeta = fq_sponge.challenge();
        let zeta2 = oracles.zeta.pow(&[n as u64]);
        let zeta3 = zeta2.square();

        // evaluate the polynomials
        evals.l = l.evaluate(oracles.zeta);
        evals.r = r.evaluate(oracles.zeta);
        evals.o = o.evaluate(oracles.zeta);
        evals.sigma1 = index.cs.sigmam[0].evaluate(oracles.zeta);
        evals.sigma2 = index.cs.sigmam[1].evaluate(oracles.zeta);
        evals.z = z.evaluate(oracles.zeta * &index.cs.domain.d1.group_gen);

        // compute linearization polynomial

        let bz = oracles.beta * &oracles.zeta;
        let f1 =
            &(&(&(&index.cs.qm.scale(evals.l*&evals.r) +
            &index.cs.ql.scale(evals.l)) +
            &index.cs.qr.scale(evals.r)) +
            &index.cs.qo.scale(evals.o)) +
            &index.cs.qc;
        let f2 =
            z.scale
            (
                (evals.l + &bz + &oracles.gamma) *
                &(evals.r + &(bz * &index.cs.r) + &oracles.gamma) *
                &(evals.o + &(bz * &index.cs.o) + &oracles.gamma) *
                &oracles.alpha +
                &(alpsq * &(zeta2 - &E::Fr::one()) / &(oracles.zeta - &E::Fr::one()))
            );
        let f3 =
            index.cs.sigmam[2].scale
            (
                (evals.l + &(oracles.beta * &evals.sigma1) + &oracles.gamma) *
                &(evals.r + &(oracles.beta * &evals.sigma2) + &oracles.gamma) *
                &(oracles.beta * &evals.z * &oracles.alpha)
            );
        let f = &(&f1 + &f2) - &f3;
        evals.f = f.evaluate(oracles.zeta);

        // query opening scaler challenge
        oracles.v = fq_sponge.challenge();

        Ok(Self
        {
            l_comm,
            r_comm,
            o_comm,
            z_comm,
            tlow_comm,
            tmid_comm,
            thgh_comm,
            proof1: index.urs.get_ref().open
            (
                vec!
                [
                    &(&(&tlow + &tmid.scale(zeta2)) + &thgh.scale(zeta3)),
                    &f,
                    &l,
                    &r,
                    &o,
                    &index.cs.sigmam[0],
                    &index.cs.sigmam[1],
                ],
                oracles.v,
                oracles.zeta
            )?,
            proof2: index.urs.get_ref().open(vec![&z], oracles.v, oracles.zeta * &index.cs.domain.d1.group_gen)?,
            evals,
            public
        })
    }
}

#[derive(Clone)]
pub struct ProofEvaluations<F> {
    pub l: F,
    pub r: F,
    pub o: F,
    pub sigma1: F,
    pub sigma2: F,
    pub f: F,
    pub z: F,
}

impl<F: Field> ProofEvaluations<F>
{
    pub fn zero () -> Self
    {
        Self
        {
            l: F::zero(),
            r: F::zero(),
            o: F::zero(),
            sigma1: F::zero(),
            sigma2: F::zero(),
            f: F::zero(),
            z: F::zero(),
        }
    }
}

pub struct RandomOracles<F: Field>
{
    pub beta: F,
    pub gamma: F,
    pub alpha: F,
    pub zeta: F,
    pub v: F,
}

impl<F: Field> RandomOracles<F>
{
    pub fn zero () -> Self
    {
        Self
        {
            beta: F::zero(),
            gamma: F::zero(),
            alpha: F::zero(),
            zeta: F::zero(),
            v: F::zero(),
        }
    }
}