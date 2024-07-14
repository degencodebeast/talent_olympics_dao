// const memberPublicKey = new PublicKey("..."); // The member's public key
// const [memberStatePDA] = await PublicKey.findProgramAddress(
//   [Buffer.from("member"), configPDA.toBuffer(), memberPublicKey.toBuffer()],
//   programId
// );

// const memberState = await program.methods.getMemberState()
//   .accounts({
//     member: memberPublicKey,
//     memberState: memberStatePDA,
//     config: configPDA,
//   })
//   .view();

// console.log(memberState);