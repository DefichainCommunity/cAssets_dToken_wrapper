import { ethers } from "https://cdn.jsdelivr.net/npm/ethers/dist/ethers.min.js";

let provider;
let signer;

export async function connect_metamask() {
    if (!window.ethereum) throw new Error("MetaMask not installed");
    await window.ethereum.request({ method: 'eth_requestAccounts' });
    provider = new ethers.BrowserProvider(window.ethereum);
    signer = await provider.getSigner();
    return await signer.getAddress();
}

export async function wrap_tokens(contractAddress, underlying, amount) {
    const abi = ["function wrap(address underlying, uint256 amount) external"];
    const contract = new ethers.Contract(contractAddress, abi, signer);
    const tx = await contract.wrap(underlying, amount);
    return tx.wait();
}

export async function unwrap_tokens(contractAddress, wrapped, amount) {
    const abi = ["function unwrap(address wrapped, uint256 amount) external"];
    const contract = new ethers.Contract(contractAddress, abi, signer);
    const tx = await contract.unwrap(wrapped, amount);
    return tx.wait();
}
