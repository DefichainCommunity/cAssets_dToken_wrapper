//import { ethers } from "./ethers.min.js";
import { ethers } from "https://cdn.jsdelivr.net/npm/ethers/dist/ethers.min.js";

// const provider;
// let signer;

const erc20Abi = [
    "function symbol() view returns (string)",
    "function decimals() view returns (uint8)",
    "function approve(address spender, uint256 amount) external returns (bool)",
    "function allowance(address owner, address spender) external view returns (uint256)",
    "function balanceOf(address) view returns (uint256)"
];

// Wrapper ABI
const wrapperAbi = [
    "function info() view returns (tuple(address dTokenAddress, address cAssetAddress, uint8 dTokenDecimals, uint8 cAssetDecimals, uint256 dTokenInFeeBps, uint256 dTokenOutFeeBps))"
];
// Wrapper Factory ABI
const wrapperFactoryAbi = ["function getAllWraps() view returns (address[])"];
// Wrapper Router ABI
const wrapperRouterAbi = [
    "function wrap(address dTokent, uint256 amount, address cAsset) external",
    "function unwrap(address cAsset, uint256 amount, address dToken) external"
];

// Uniswap V2
const uniswapV2RouterAbi = [
    "function factory() view returns (address)",
    "function swapExactTokensForTokensSupportingFeeOnTransferTokens(uint256 amountIn, uint256 amountOutMin, address[] calldata path, address to, uint256 deadline) returns (uint256[])",
    "function swapExactTokensForETHSupportingFeeOnTransferTokens(uint256 amountIn, uint256 amountOutMin, address[] calldata path, address to, uint256 deadline)",
    "function swapExactETHForTokensSupportingFeeOnTransferTokens(uint256 amountOutMin, address[] calldata path, address to, uint256 deadline) payable",
    "function addLiquidity(address tokenA, address tokenB, uint amountADesired, uint amountBDesired, uint amountAMin, uint amountBMin, address to, uint deadline) external returns (uint amountA, uint amountB, uint liquidity)",
    "function addLiquidityETH(address token, uint amountTokenDesired, uint amountTokenMin, uint amountETHMin, address to, uint deadline) payable returns (uint amountToken, uint amountETH, uint liquidity)",
    "function removeLiquidity(address tokenA, address tokenB, uint liquidity, uint amountAMin, uint amountBMin, address to, uint deadline) external returns (uint amountA, uint amountB)",
    "function removeLiquidityETH(address token, uint liquidity, uint amountTokenMin, uint amountETHMin, address to, uint deadline) external returns (uint amountToken, uint amountETH)"
];
const uniswapV2FactoryAbi = [
    "function allPairsLength() view returns (uint256)",
    "function allPairs(uint256) view returns (address)"
];
const uniswapV2PairAbi = [
    "function token0() view returns (address)",
    "function token1() view returns (address)",
    "function getReserves() view returns (uint112,uint112,uint32)"
];

// Uniswap V3
const uniswapV3PairAbi = [ "function slot0() view returns (uint160 sqrtPriceX96, int24, uint16, uint16, uint16, uint8, bool)", "function liquidity() view returns (uint128)" ];

const uniswapV3RouterAbi = [
    "function exactInput((bytes path,address recipient,uint256 amountIn,uint256 amountOutMinimum)) external payable returns (uint256 amountOut)"
];
const wethAbi = [
  "function withdraw(uint256 wad) external",
  "function balanceOf(address owner) view returns (uint256)"
];

/**
 * helper JSON stringify that converts BigInt to string
 */
function safeStringify(obj) {
  return JSON.stringify(obj, (k, v) =>
    typeof v === "bigint" ? v.toString() : v
  );
}

// METAMASK
export async function js_connect_metamask() {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);
        let signer = await provider.getSigner();
        const addr = await signer.getAddress();
        const network = await provider.getNetwork();
        //console.log("Address", addr)
        // console.log("ChainId", network.chainId)
        return {
            ok: true,
            value: safeStringify({
                address: addr,
                chainId: network.chainId
            })
        };
    }catch (err) {
        console.error("Metamask connect error:",err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }
}

export function js_on_chain_changed(callback) {
    if (window.ethereum) {
        window.ethereum.on('chainChanged', (chainId) => {
            const numericChainId = parseInt(chainId, 16);
            callback(numericChainId);
        });
    }
}

export function js_on_accounts_changed(callback) {
    if (window.ethereum) {
        window.ethereum.on('accountsChanged', (accounts) => {
            callback(accounts);
        });
    }
}

// ERC20
export async function js_get_token_balance(user, token, isNative) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);

        let bal, decimals;

        if (isNative) {
            [bal, decimals] = await Promise.all([provider.getBalance(user), 18]);
        }else{
            const erc20 = new ethers.Contract(token, erc20Abi, provider);
            [bal, decimals] = await Promise.all([erc20.balanceOf(user), erc20.decimals()]);
        }
        return {
            ok: true,
            value: JSON.stringify(ethers.formatUnits(bal, decimals))
        };
    } catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }
}

export async function js_get_tokens_balances(user, tokens) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);

        const calls = tokens.map(async token => {
            const c = new ethers.Contract(token, erc20Abi, provider);
            const bal = await c.balanceOf(user);
            return [token, bal.toString()];
        });
        const results = await Promise.all(calls);
        const bal = await provider.getBalance(user);
        // convert to object mapping
        const map = {};
        map['native'] = bal;
        for (const [addr, value] of results) {
            map[addr] = value;
        }

        return {
            ok: true,
            value: safeStringify(map)
        };
    } catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }
}


// FACTORY
export async function js_get_all_wrappers(factoryAddress) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);

        // Factory ABI
        const factory = new ethers.Contract(factoryAddress, wrapperFactoryAbi, provider);

        const wrapAddresses = await factory.getAllWraps();

        // 1. Fetch all wrapper infos in parallel
        const wrapperInfos = await Promise.all(
            wrapAddresses.map(async (wrapperAddr) => {
                try {
                    const wrapper = new ethers.Contract(wrapperAddr, wrapperAbi, provider);
                    const info = await wrapper.info();
                    return { wrapperAddr, info };
                } catch (err) {
                    console.warn("Skipping wrapper", wrapperAddr, err);
                    return null;
                }
            })
        );

        // Filter out nulls
        const validWrappers = wrapperInfos.filter(Boolean);

        // 2. Deduplicate token addresses
        const uniqueTokens = [
            ...new Set(validWrappers.flatMap(w => [w.info.dTokenAddress, w.info.cAssetAddress]))
        ];

        const symbolMap = {};
        await Promise.all(uniqueTokens.map(async (addr) => {
            try {
                const token = new ethers.Contract(addr, erc20Abi, provider);
                symbolMap[addr] = await token.symbol();
            } catch (_) {
                symbolMap[addr] = null;
            }
        }));

        // 4. Build final token list
        const tokenList = validWrappers.map(({ wrapperAddr, info }) => ({
            wrapper: wrapperAddr,
            dTokenSymbol: symbolMap[info.dTokenAddress] ?? null,
            dTokenAddress: info.dTokenAddress,
            dTokenDecimals: info.dTokenDecimals,
            cAssetSymbol: symbolMap[info.cAssetAddress] ?? null,
            cAssetAddress: info.cAssetAddress,
            cAssetDecimals: info.cAssetDecimals,
            fees: {
                inBps: info.dTokenInFeeBps,
                outBps: info.dTokenOutFeeBps
            }
        }));
        console.log(safeStringify(tokenList));
        return { ok: true, value: safeStringify(tokenList) };
    } catch (err) {
        return { ok: false, value: err.reason || err.message || "Unknown error" };
    }
}

// ROUTER
export async function js_wrap_tokens(contractAddress, dToken, amount, cAsset) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);
        let signer = await provider.getSigner();

        console.log("contractAddress:", contractAddress, " dToken:",dToken, " Amount:",amount," cAsset:", cAsset);
        const erc20_contract = new ethers.Contract(dToken, erc20Abi, signer);
        const decimals = await erc20_contract.decimals();
        const amount_u256 = ethers.parseUnits(amount,decimals);
        const erc20_connected = erc20_contract.connect(signer);
        const allowance = await erc20_connected.allowance(signer, contractAddress);
        if (allowance < amount_u256){
            const approve_tx = await erc20_connected.approve(contractAddress, amount_u256);
            await approve_tx.wait();
        }
        const contract = new ethers.Contract(contractAddress, wrapperRouterAbi, signer);
        const connected = contract.connect(signer);
        const tx = await connected.wrap(dToken, amount_u256, cAsset);
        const receipt = await tx.wait();
        return {
            ok: true,
            value: JSON.stringify(`${receipt.hash}`)
        };
    } catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }

}

export async function js_unwrap_tokens(contractAddress, cAsset, amount, dToken) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);
        let signer = await provider.getSigner();

        console.log("Unwrap on contractAddress:", contractAddress, " cAsset:", cAsset, " Amount:",amount, " dToken:",dToken);
        const erc20_contract = new ethers.Contract(cAsset, erc20Abi, signer);
        const decimals = await erc20_contract.decimals();
        const amount_u256 = ethers.parseUnits(amount,decimals);
        const erc20_connected = erc20_contract.connect(signer);
        const allowance = await erc20_connected.allowance(signer, contractAddress);
        if (allowance < amount_u256){
            const approve_tx = await erc20_connected.approve(contractAddress, amount_u256);
            await approve_tx.wait();
        }
        const contract = new ethers.Contract(contractAddress, wrapperRouterAbi, signer);
        const connected = contract.connect(signer);
        const tx = await connected.unwrap(cAsset, amount_u256, dToken);
        const receipt = await tx.wait();
        return {
            ok: true,
            value: JSON.stringify(`${receipt.hash}`)
        };
    } catch (err) {
        console.error(err);
        return {
            ok: false,
            value: err.reason || err.message || "Unknown error"
        };
    }
}


//UniSwap V2
export async function js_get_uniswap_v2_pairs(routerAddr) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);
        // 1. Get factory address
        const router = new ethers.Contract(routerAddr, uniswapV2RouterAbi, provider);
        const factoryAddr = await router.factory();
        const factory = new ethers.Contract(factoryAddr, uniswapV2FactoryAbi, provider);

        // 2. Get number of pairs
        const len = Number(await factory.allPairsLength());

        // 3. Fetch all pair addresses in parallel
        const pairAddrs = await Promise.all(
            Array.from({ length: len }, (_, i) => factory.allPairs(i))
        );

        // 4. Fetch token0/token1/reserves for all pairs in parallel
        const tokenPairs = await Promise.all(
            pairAddrs.map(async (pairAddr) => {
                try {
                    const pair = new ethers.Contract(pairAddr, uniswapV2PairAbi, provider);
                    const [token0, token1] = await Promise.all([pair.token0(), pair.token1()]);

                    let reserve0 = null, reserve1 = null;
                    try {
                        const r = await pair.getReserves();
                        reserve0 = r[0].toString();
                        reserve1 = r[1].toString();
                    } catch (_) {}

                    return { pairAddr, token0, token1, reserve0, reserve1 };
                } catch (_) {
                    return { pairAddr, token0: null, token1: null, reserve0: null, reserve1: null };
                }
            })
        );

        // 5. Deduplicate tokens to minimize symbol/decimals calls
        const uniqueTokens = [...new Set(tokenPairs.flatMap(p => [p.token0, p.token1]).filter(Boolean))];
        const tokenInfoMap = {};

        await Promise.all(
            uniqueTokens.map(async (tokenAddr) => {
                try {
                    const token = new ethers.Contract(tokenAddr, erc20Abi, provider);
                    const [symbol, decimals] = await Promise.all([
                        token.symbol().catch(() => null),
                        token.decimals().catch(() => null)
                    ]);
                    tokenInfoMap[tokenAddr] = { symbol, decimals };
                } catch (_) {
                    tokenInfoMap[tokenAddr] = { symbol: null, decimals: null };
                }
            })
        );

        // 6. Build final pairs array
        const pairs = tokenPairs.map(p => ({
            pair_address: p.pairAddr,
            token0: p.token0,
            token1: p.token1,
            symbol0: tokenInfoMap[p.token0]?.symbol ?? null,
            symbol1: tokenInfoMap[p.token1]?.symbol ?? null,
            decimals0: tokenInfoMap[p.token0]?.decimals ?? null,
            decimals1: tokenInfoMap[p.token1]?.decimals ?? null,
            reserve0: p.reserve0,
            reserve1: p.reserve1
        }));

        return { ok: true, value: safeStringify(pairs) };
    } catch (err) {
        return { ok: false, value: err?.message || String(err) };
    }
}

export async function js_uniswap_v2_swap_tokens(tokenIn, tokenOut, amountIn, amountOutMin, routerAddress, isNativeIn, isNativeOut) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);
        let signer = await provider.getSigner();

        const router = new ethers.Contract(routerAddress, uniswapV2RouterAbi, signer);

        const tokenInContract = new ethers.Contract(tokenIn, erc20Abi, signer);
        const decimals = await tokenInContract.decimals();
        const amount_in_u256 = amountIn;

        const path = [tokenIn, tokenOut];
        const deadline = Math.floor(Date.now() / 1000) + 60 * 10;

        let tx;
        if (isNativeIn){
            tx = await router.swapExactETHForTokensSupportingFeeOnTransferTokens(
                amountOutMin,
                path,
                await signer.getAddress(),
                deadline,
                {
                    value: amountIn
                }
            );
        }else{
            // Approve
            const allowance = await tokenInContract.allowance(signer, routerAddress);
            if (allowance < amount_in_u256){
                const approve_tx = await tokenInContract.approve(routerAddress, amount_in_u256);
                await approve_tx.wait();
            }


            if (isNativeOut){
                tx = await router.swapExactTokensForETHSupportingFeeOnTransferTokens(
                    amountIn,
                    amountOutMin,
                    path,
                    await signer.getAddress(),
                    deadline,
                );
            }else{
                tx = await router.swapExactTokensForTokensSupportingFeeOnTransferTokens(
                    amountIn,
                    amountOutMin,
                    path,
                    await signer.getAddress(),
                    deadline,
                );
            }

        }
        const receipt = await tx.wait();

        return {
            ok: true,
            value: JSON.stringify(`${receipt.hash}`)
        };
    } catch (err) {
        console.error(err);
        return { ok: false, value: err?.reason || err?.message || String(err) };
    }
}

export async function js_uniswap_v2_add_liquidity(tokenA, tokenB, amountA, amountB, routerAddress, isNativeA, isNativeB) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);
        let signer = await provider.getSigner();

        const router = new ethers.Contract(routerAddress, uniswapV2RouterAbi, signer);

        if (!isNativeA){
            const tokenAContract = new ethers.Contract(tokenA, erc20Abi, signer);
            // Approve
            const allowance = await tokenAContract.allowance(signer, routerAddress);
            if (allowance < amountA){
                const approveATx = await tokenAContract.approve(routerAddress, amountA);
                await approveATx.wait();
            }
        }
        if (!isNativeB){
            const tokenBContract = new ethers.Contract(tokenB, erc20Abi, signer);
            // Approve
            const allowance = await tokenBContract.allowance(signer, routerAddress);
            if (allowance < amountB){
                const approveBTx = await tokenBContract.approve(routerAddress, amountB);
                await approveBTx.wait();
            }
        }

        const deadline = Math.floor(Date.now() / 1000) + 60 * 10;

        let tx;
        if (isNativeA || isNativeB){
            const tokenAddress = isNativeA ? tokenB : tokenA;
            const tokenAmount  = isNativeA ? amountB : amountA;
            const nativeAmount  = isNativeA ? amountA : amountB;
            tx = await router.addLiquidityETH(
                tokenAddress,
                tokenAmount,
                0,
                await signer.getAddress(), // TODO: use just signer
                deadline,
                {
                    value: nativeAmount
                }
            );
        }else{
            tx = await router.addLiquidity(
                tokenA,
                tokenB,
                amountA,
                amountB,
                0,
                0,
                await signer.getAddress(),
                deadline,
            );
        }
        const receipt = await tx.wait();

        return {
            ok: true,
            value: JSON.stringify(`${receipt.hash}`)
        };
    } catch (err) {
        console.error(err);
        return { ok: false, value: err?.reason || err?.message || String(err) };
    }
}


/// UniSwap V3
export async function js_get_uniswap_v3_pool_states(poolAddrs) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);

        // build all calls in parallel
        const results = await Promise.all(poolAddrs.map(async (poolAddr) => {
            try {
                const pool = new ethers.Contract(poolAddr, uniswapV3PairAbi, provider);

                // parallel call slot0 + liquidity
                const [slot0, liquidity] = await Promise.all([pool.slot0(), pool.liquidity()]);

                return [poolAddr, {
                    sqrtPriceX96: slot0.sqrtPriceX96.toString(),
                    liquidity: liquidity.toString()
                }];
            } catch (err) {
                console.warn("Error fetching pool", poolAddr, err);
                return [poolAddr, null];
            }
        }));

        // convert to object mapping
        const map = {};
        for (const [poolAddr, value] of results) {
            map[poolAddr] = value;
        }

        return { ok: true, value: safeStringify(map) };

    } catch (err) {
        console.error(err);
        return { ok: false, value: err?.reason || err?.message || String(err) };
    }
}

function encodePath(tokenIn, fee, tokenOut) {
    return ethers.concat([
        ethers.getAddress(tokenIn),
        ethers.toBeHex(fee, 3),
        ethers.getAddress(tokenOut)
    ]);
}

export async function js_uniswap_v3_swap_tokens(tokenIn, tokenOut, amountIn, amountOutMin, fee, routerAddress, isNativeIn, isNativeOut) {
    try {
        if (!window.ethereum) throw new Error("MetaMask not installed");
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        let provider = new ethers.BrowserProvider(window.ethereum);
        let signer = await provider.getSigner();


        const router = new ethers.Contract(routerAddress, uniswapV3RouterAbi, signer);

        const tokenInContract = new ethers.Contract(tokenIn, erc20Abi, signer);
        const decimals = await tokenInContract.decimals();
        const amount_in_u256 = amountIn;//thers.parseUnits(amountIn,decimals);

        // Approve
        const allowance = await tokenInContract.allowance(signer, routerAddress);
        if (allowance < amount_in_u256){
            const approve_tx = await tokenInContract.approve(routerAddress, amount_in_u256);
            await approve_tx.wait();
        }

        const path = encodePath(tokenIn,fee,tokenOut);
        const recipient = await signer.getAddress();
        const params = {
            path:path,
            recipient:recipient,
            amountIn:amount_in_u256,
            amountOutMinimum: amountOutMin
        };
        const tx = await router.exactInput(params,isNativeIn ? { value: amountIn } : {});
        const receipt = await tx.wait();
        if (isNativeOut) {
            const wethContract = new ethers.Contract(tokenOut, wethAbi, signer);
            const balance = await wethContract.balanceOf(recipient);
            if (balance > 0n) {
                const unwrapTx = await wethContract.withdraw(balance);
                await unwrapTx.wait();
            }
        }
        return {
            ok: true,
            value: JSON.stringify(`${receipt.hash}`)
        };
    } catch (err) {
        console.error(err);
        return { ok: false, value: err?.reason || err?.message || String(err) };
    }
}
