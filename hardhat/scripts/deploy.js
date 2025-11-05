// scripts/deploy.js
async function main() {
  const HelloWorld = await ethers.getContractFactory("HelloWorld");
  const hello_world = await HelloWorld.deploy("Hello World!");
  console.log("Contract deployed to address:", hello_world.address);
}
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});