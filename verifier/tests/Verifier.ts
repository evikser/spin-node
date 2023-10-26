import { expect } from "chai";
import { ethers } from "hardhat";
import { readFileSync } from 'fs';
import { sha256 } from "ethers";

describe("Onchain Verify", function () {

  it("Fibonacci Verification", async function () {
    const [owner] = await ethers.getSigners();

    const verifier = await ethers.deployContract("RiscZeroGroth16Verifier");

    // const path = '/Users/nikita/Develop/spin-node-evikser/';
    const path = '/Users/nikita/Develop/spin-node-evikser/simple_contract_state/';
    const seal = readFileSync(path + 'seal.txt', 'utf-8');
    const imageId = readFileSync(path + 'imageId.txt', 'utf-8');
    const postStateDigest = readFileSync(path + 'postStateDigest.txt', 'utf-8');
    const journalHash = readFileSync(path + 'journalHash.txt', 'utf-8');

    const result = await verifier['verify(bytes,bytes32,bytes32,bytes)'](seal, imageId, postStateDigest, journalHash);
    console.log(result);
  });

  // it("Bonsai Governor Test Verification", async function () {
  //   const [owner] = await ethers.getSigners();

  //   const verifier = await ethers.deployContract("RiscZeroGroth16Verifier");

  //   const seal = "0x0ca9d14477c5ac35d4bcc3562f3e2b70f52696d82af496fea385fc4c997ffbf013696413e37acb3c6ec156135f3083fd4238644a237b95e6702f222df5c012082d5e0e6e86f557a2c5385af338f81b75c33b18e93ebf152eaaf3639ee3edc24815070be3b40806c5ed815ea0d58a1eb3d44ae2c56a91a0122dd006ad030129d126d852777e5e418bad8dc374c5743350206f0b0d1d79adfeca47636788aad94d2b4a78e87ac2ae3d7bc291a6f2c08047ea0534936ca3891fc6ff068e0ccd5c2a25f6f43e7ffecaf29e8fc6a872a98c2b7a273d3d3b987e589ca9830def18491f2f27576b784b8de4020edbbc1c2fb560e17f7bf622ca543336b29eefae33d0a5"
  //   const imageId = "0x0277b0302c0f8f30dbf65997178ee700a7af30d5512796bcabf02996f9022b4d";
  //   const postStateDigest = "0xbab5928deaa6b9f89acecdc86a4b9f20fdf5bb5b631780cf67e7ad0afeace872";
  //   const journalHash = "0x5818100a2105c60d4f73044fe09a9cb0ba9801a4f5775e79cbb8934b23caab653c7846705db9354810f597a10674ad845f1a11d31cdd54fa7ca011ebf45c67000000000040eb306043ba7f507c09693f6d68f07f50722b010000000142add52666c78960a219b157a1f4dbf806cbf703";

  //   const result = await verifier['verify(bytes,bytes32,bytes32,bytes)'](seal, imageId, postStateDigest, journalHash);
  //   console.log(result);
  // });

  it("Bonsai Fibonacci Test Verification", async function () {
    const [owner] = await ethers.getSigners();

    const verifier = await ethers.deployContract("RiscZeroGroth16Verifier");

    const seal = "0x29c458c767b4875b439e6ea7eae08b4042ba3552d05a4cef418d614a425dc93c217a4832bbc588e9a223bf05944b011d24782f32f22499823c086f262c77cb790fc3160308a79173d72937bdd8a0a407cb246b58e8e2df99f4796435fc214ed920b7103ab2f5d934c64be460d8ca882ea1c0043120d38ecb5d7962923e3051bd12409bf33479595903d626da776541dcc4c9732743779ec17c44db21a38b965b28071b21a65209dd23959abac6bec998ab38b8e44ab3678b931bd610ca50bf920d89803ab46a3829bc910810e5d0c18e776db8c42f6e6e012d0c6a9f107ff8010e00bc47da01aa573176dfacffae74fff6d107419cd461ec9240e00bdfc1219d"
    const imageId = "0xbe1c1c8a02bb78a05e294ade5ed1bf37cd3cbd0751053204fc4274eaa76b497a";
    const postStateDigest = "0x1ead55c6871eafce7ac0174f0e214d1db787760017d591c3e90eaf82722232d3";
    const journalHash = "0x420000003078333265336266313036366533646166616161343462313733613835333333656137373661646138623238303134326435306561333965333230353133313935630000";

    const result = await verifier['verify(bytes,bytes32,bytes32,bytes)'](seal, imageId, postStateDigest, journalHash);
    console.log(result);
  });
});