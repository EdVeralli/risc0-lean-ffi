import LeanVerifier.Verifier

def main : IO Unit := do
  IO.println "=== Lean Verifier Test ==="

  let value : UInt64 := 42
  let nonce : UInt64 := 12345

  let commitment := createCommitment value nonce
  IO.println s!"Commitment de {value} con nonce {nonce}: {commitment}"

  let valid := verifyCommitment commitment value nonce
  IO.println s!"Verificación: {valid}"

  let invalid := verifyCommitment commitment 99 nonce
  IO.println s!"Verificación con valor incorrecto: {invalid}"
