//! This module defines various traits required by the users of the library to implement.
use bellperson::{
  gadgets::{boolean::AllocatedBit, num::AllocatedNum},
  ConstraintSystem, SynthesisError,
};
use core::{
  fmt::Debug,
  ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};
use ff::{PrimeField, PrimeFieldBits};
use merlin::Transcript;
use num_bigint::BigInt;

/// Represents an element of a group
pub trait Group:
  Clone
  + Copy
  + Debug
  + Eq
  + Sized
  + GroupOps
  + GroupOpsOwned
  + ScalarMul<<Self as Group>::Scalar>
  + ScalarMulOwned<<Self as Group>::Scalar>
  + Send
  + Sync
{
  /// A type representing an element of the base field of the group
  type Base: PrimeField + PrimeFieldBits;

  /// A type representing an element of the scalar field of the group
  type Scalar: PrimeField + PrimeFieldBits + ChallengeTrait + Send + Sync;

  /// A type representing the compressed version of the group element
  type CompressedGroupElement: CompressedGroup<GroupElement = Self>;

  /// A type representing preprocessed group element
  type PreprocessedGroupElement: Clone + Send + Sync;

  /// A type that represents a hash function that consumes elements
  /// from the base field and squeezes out elements of the scalar field
  type HashFunc: HashFuncTrait<Self::Base, Self::Scalar>;

  /// An alternate implementation of Self::HashFunc in the circuit model
  type HashFuncCircuit: HashFuncCircuitTrait<Self::Base>;

  /// A method to compute a multiexponentation
  fn vartime_multiscalar_mul(
    scalars: &[Self::Scalar],
    bases: &[Self::PreprocessedGroupElement],
  ) -> Self;

  /// Compresses the group element
  fn compress(&self) -> Self::CompressedGroupElement;

  /// Produces a preprocessed element
  fn preprocessed(&self) -> Self::PreprocessedGroupElement;

  /// Produce a vector of group elements using a static label
  fn from_label(label: &'static [u8], n: usize) -> Vec<Self::PreprocessedGroupElement>;

  /// Returns the affine coordinates (x, y, infinty) for the point
  fn to_coordinates(&self) -> (Self::Base, Self::Base, bool);

  /// Returns the order of the group as a big integer
  fn get_order() -> BigInt;
}

/// Represents a compressed version of a group element
pub trait CompressedGroup: Clone + Copy + Debug + Eq + Sized + Send + Sync + 'static {
  /// A type that holds the decompressed version of the compressed group element
  type GroupElement: Group;

  /// Decompresses the compressed group element
  fn decompress(&self) -> Option<Self::GroupElement>;

  /// Returns a byte array representing the compressed group element
  fn as_bytes(&self) -> &[u8];
}

/// A helper trait to append different types to the transcript
pub trait AppendToTranscriptTrait {
  /// appends the value to the transcript under the provided label
  fn append_to_transcript(&self, label: &'static [u8], transcript: &mut Transcript);
}

/// A helper trait to absorb different objects in RO
pub trait AbsorbInROTrait<G: Group> {
  /// Absorbs the value in the provided RO
  fn absorb_in_ro(&self, ro: &mut G::HashFunc);
}

/// A helper trait to generate challenges using a transcript object
pub trait ChallengeTrait {
  /// Returns a Scalar representing the challenge using the transcript
  fn challenge(label: &'static [u8], transcript: &mut Transcript) -> Self;
}

/// A helper trait that defines the behavior of a hash function that we use as an RO
pub trait HashFuncTrait<Base, Scalar> {
  /// A type representing constants/parameters associated with the hash function
  type Constants: HashFuncConstantsTrait<Base> + Clone + Send + Sync;

  /// Initializes the hash function
  fn new(constants: Self::Constants) -> Self;

  /// Adds a scalar to the internal state
  fn absorb(&mut self, e: Base);

  /// Returns a random challenge by hashing the internal state
  fn get_challenge(&self) -> Scalar;

  /// Returns a hash of the internal state
  fn get_hash(&self) -> Scalar;
}

/// A helper trait that defines the behavior of a hash function that we use as an RO in the circuit model
pub trait HashFuncCircuitTrait<Base: PrimeField> {
  /// A type representing constants/parameters associated with the hash function
  type Constants: HashFuncConstantsTrait<Base> + Clone + Send + Sync;

  /// Initializes the hash function
  fn new(constants: Self::Constants) -> Self;

  /// Adds a scalar to the internal state
  fn absorb(&mut self, e: AllocatedNum<Base>);

  /// Returns a random challenge by hashing the internal state
  fn get_challenge<CS>(&mut self, cs: CS) -> Result<Vec<AllocatedBit>, SynthesisError>
  where
    CS: ConstraintSystem<Base>;

  /// Returns a hash of the internal state
  fn get_hash<CS>(&mut self, cs: CS) -> Result<Vec<AllocatedBit>, SynthesisError>
  where
    CS: ConstraintSystem<Base>;
}

/// A helper trait that defines the constants associated with a hash function
pub trait HashFuncConstantsTrait<Base> {
  /// produces constants/parameters associated with the hash function
  fn new() -> Self;
}

/// An alias for constants associated with G::HashFunc
pub type HashFuncConstants<G> =
  <<G as Group>::HashFunc as HashFuncTrait<<G as Group>::Base, <G as Group>::Scalar>>::Constants;

/// An alias for constants associated with G::HashFuncCircuit
pub type HashFuncConstantsCircuit<G> =
  <<G as Group>::HashFuncCircuit as HashFuncCircuitTrait<<G as Group>::Base>>::Constants;

/// A helper trait for types with a group operation.
pub trait GroupOps<Rhs = Self, Output = Self>:
  Add<Rhs, Output = Output> + Sub<Rhs, Output = Output> + AddAssign<Rhs> + SubAssign<Rhs>
{
}

impl<T, Rhs, Output> GroupOps<Rhs, Output> for T where
  T: Add<Rhs, Output = Output> + Sub<Rhs, Output = Output> + AddAssign<Rhs> + SubAssign<Rhs>
{
}

/// A helper trait for references with a group operation.
pub trait GroupOpsOwned<Rhs = Self, Output = Self>: for<'r> GroupOps<&'r Rhs, Output> {}
impl<T, Rhs, Output> GroupOpsOwned<Rhs, Output> for T where T: for<'r> GroupOps<&'r Rhs, Output> {}

/// A helper trait for types implementing group scalar multiplication.
pub trait ScalarMul<Rhs, Output = Self>: Mul<Rhs, Output = Output> + MulAssign<Rhs> {}

impl<T, Rhs, Output> ScalarMul<Rhs, Output> for T where T: Mul<Rhs, Output = Output> + MulAssign<Rhs>
{}

/// A helper trait for references implementing group scalar multiplication.
pub trait ScalarMulOwned<Rhs, Output = Self>: for<'r> ScalarMul<&'r Rhs, Output> {}
impl<T, Rhs, Output> ScalarMulOwned<Rhs, Output> for T where T: for<'r> ScalarMul<&'r Rhs, Output> {}

impl<F: PrimeField> AppendToTranscriptTrait for F {
  fn append_to_transcript(&self, label: &'static [u8], transcript: &mut Transcript) {
    transcript.append_message(label, self.to_repr().as_ref());
  }
}

impl<F: PrimeField> AppendToTranscriptTrait for [F] {
  fn append_to_transcript(&self, label: &'static [u8], transcript: &mut Transcript) {
    for s in self {
      s.append_to_transcript(label, transcript);
    }
  }
}

pub mod circuit;
pub mod snark;
