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

export async function wrap_tokens(contractAddress, dToken, amount, cAsset) {
    try {
        const abi = ["function wrap(address dTokent, uint256 amount, address cAsset) external"];
        const approveAbi = ["function approve(address spender, uint256 amount) external returns (bool)"];
        console.log("contractAddress:", contractAddress, " dToken:",dToken, " Amount:",amount," cAsset:", cAsset);
        const erc20_contract = new ethers.Contract(dToken, approveAbi, signer);
        const erc20_connected = erc20_contract.connect(signer);
        const approve_tx = await erc20_connected.approve(contractAddress, amount);
        await approve_tx.wait();
        const contract = new ethers.Contract(contractAddress, abi, signer);
        const connected = contract.connect(signer);
        const tx = await connected.wrap(dToken, amount, cAsset);
        await tx.wait();
        return "Transaction successful";
    } catch (err) {
        console.error(err);
        return `Error: ${err.reason || err.message}`;
    }

}

export async function unwrap_tokens(contractAddress, cAsset, amount, dToken) {
    try {
        const abi = ["function unwrap(address cAsset, uint256 amount, address dToken) external"];
        const approveAbi = ["function approve(address spender, uint256 amount) external returns (bool)"];
        console.log("contractAddress:", contractAddress, " cAsset:", cAsset, " Amount:",amount, " dToken:",dToken);
        const erc20_contract = new ethers.Contract(cAsset, approveAbi, signer);
        const erc20_connected = erc20_contract.connect(signer);
        const approve_tx = await erc20_connected.approve(contractAddress, amount);
        await approve_tx.wait();
        const contract = new ethers.Contract(contractAddress, abi, signer);
        const connected = contract.connect(signer);
        const tx = await connected.unwrap(cAsset, amount, dToken);
        await tx.wait();
        return "Transaction successful";
    } catch (err) {
        console.error(err);
        return `Error: ${err.reason || err.message}`;
    }
}
