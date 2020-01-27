import * as env from "./env";
import { Encoding, getDataPtr } from "./utils";

/**
 * Refers to some heap allocated data in wasm.
 * A pointer to this can be returned over ffi boundaries.
 */
@unmanaged
export class Region {
  offset: u32;
  len: u32;
}

function readRegion(regionPtr: usize): Uint8Array {
  const region = changetype<Region>(regionPtr);

  // This copy is required because the basic binary type ArrayBuffer has an
  // AssemblyScript-specific 16 bytes "common header", which is not provided by the VM.
  // https://docs.assemblyscript.org/details/memory#internals
  const buffer = new ArrayBuffer(region.len);
  memory.copy(changetype<usize>(buffer), region.offset, region.len);

  return Uint8Array.wrap(buffer);
}

/**
 * Reserves the given number of bytes in wasm memory. Creates a Region and returns a pointer
 * to that Region.
 * This space is managed by the calling process and should be accompanied by a corresponding deallocate.
 */
export function allocate(size: usize): usize {
  const dataPtr = __alloc(size, idof<ArrayBuffer>());
  __retain(dataPtr);

  const region: Region = {
    offset: dataPtr,
    len: size,
  };
  const regionPtr = changetype<usize>(region);
  __retain(regionPtr);
  return regionPtr;
}

/**
 * Expects a pointer to a Region created with allocate.
 * It will free both the Region and the memory referenced by the Region.
 */
export function deallocate(regionPtr: usize): void {
  const dataPtr = changetype<Region>(regionPtr).offset;
  __release(regionPtr); // release Region
  __release(dataPtr); // release ArrayBuffer
}

/**
 * Releases ownership of the data without destroying it.
 */
export function releaseOwnership(data: Uint8Array): usize {
  const dataPtr = getDataPtr(data);

  const region: Region = {
    offset: dataPtr,
    len: data.byteLength,
  };
  const regionPtr = changetype<usize>(region);

  // Retain both raw data as well as the Region object referring to it
  __retain(dataPtr);
  __retain(regionPtr);

  return regionPtr;
}

/**
 * Creates a Region linking to the given data.
 * Keeps ownership of the data and the Region and returns a pointer to the Region.
 */
export function keepOwnership(data: Uint8Array): usize {
  const dataPtr = getDataPtr(data);

  const region: Region = {
    offset: dataPtr,
    len: data.byteLength,
  };
  return changetype<usize>(region);
}

/**
 * Takes ownership of the data at the given pointer
 */
export function takeOwnership(regionPtr: usize): Uint8Array {
  const out = readRegion(regionPtr);
  deallocate(regionPtr);
  return out;
}

export function log(text: string): void {
  const data = Encoding.toUtf8(text);
  env.log(keepOwnership(data));
}

export function canonicalize(human: string): Uint8Array {
  const humanEncoded = Encoding.toUtf8(human);
  const resultPtr = allocate(50);
  const returnCode = env.canonicalize_address(keepOwnership(humanEncoded), resultPtr);
  if (returnCode < 0) {
    throw new Error(
      "Call to env.canonicalize_address failed with return code " + returnCode.toString(),
    );
  }
  const canonical = readRegion(resultPtr);
  deallocate(resultPtr);
  return canonical;
}

export function logAndCrash(
  message: string | null,
  fileName: string | null,
  lineNumber: u32,
  columnNumber: u32,
): void {
  const msg =
    "Aborted with message '" +
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    (message || "unset")! +
    " (in '" +
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    (fileName || "unset")! +
    "', line " +
    lineNumber.toString() +
    ", column " +
    columnNumber.toString() +
    ")";
  log(msg);
  unreachable(); // crash hard
}
