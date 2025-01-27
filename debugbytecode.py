SIGNATURE = b'Haru//'
SIGNATURE_LEN = len(SIGNATURE)

VmOpcode = {
    'Halt': 0, 
    'Push8': 1,
    'Push16': 2,
    'Push32': 3,
    'Push64': 4,
    'PushBool': 5,
    'PushNil': 6,
    'PushStr': 7,
    'PushStrInterned': 8,
    'Pushf64': 9,
    'Pop': 10, 
    'Add': 11,
    'Sub': 12,
    'Mul': 13,
    'Div': 14,
    'Mod': 15,
    'IAdd': 16,
    'IMul': 17,
    'BitwiseAnd': 18,
    'BitwiseOr': 19,
    'BitwiseXOR': 20, 
    'Negate': 21,
    'Not': 22,
    'Lt': 23,
    'LEq': 24,
    'Gt': 25,
    'GEq': 26,
    'Eq': 27,
    'NEq': 28,
    'Of': 29, 
    'EnvNew': 30, 
    'SetLocal': 31,
    'SetLocalFunctionDef': 32,
    'GetLocal': 33,
    'GetLocalUp': 34,
    'SetGlobal': 35,
    'GetGlobal': 36,
    'DefFunctionPush': 37,
    'Jmp': 38,
    'JmpLong': 39,
    'JCond': 40, 
    'JNcond': 41,
    'Call': 42,
    'Ret': 43,
    'JCondNoPop': 44,
    'JNcondNoPop': 45,
    'DictNew': 46,
    'MemberGet': 47,
    'MemberGetNoPop': 48,
    'MemberSet': 49,
    'DictLoad': 50, 
    'ArrayLoad': 51,
    'IndexGet': 52,
    'IndexGetNoPop': 53,
    'IndexSet': 54,
    'Try': 55,
    'Raise': 56,
    'ExframeRet': 57,
    'RetCall': 58,
    'ForIn': 59,
    'Swap': 60, 
    'Use': 61,
}

VmOpcode_from_byte = {
    v: k for k, v in VmOpcode.items()
}

# print(VmOpcode_from_byte)
# exit()

# print([chr(x) for x in signature])
def print_bytes(bytes):
    print(bytes)
    for byte in bytes:
        #print("byte is: ", byte)
        if byte == b"\n":
            print()
        else:
            print(VmOpcode_from_byte[byte], ", ", end="")
            


def print_bytes_from_file(file_path, ignore_signature=True):

    try:
        with open(file_path, "rb") as file:
            bytes = file.read()
            if ignore_signature:
                doc_signature = bytes[:SIGNATURE_LEN]
                print(doc_signature == SIGNATURE)
                
                if doc_signature == SIGNATURE:
                    bytes = bytes[SIGNATURE_LEN:]
                else:
                    print("Signature mismatch.")
                    return

            print_bytes(bytes)

    except FileNotFoundError:
        print(f"File {file_path} not found.")

    except Exception as e:
        print(f"An error occurred: {e}")


# Llamada a la función con el archivo 'bt.txt'
print_bytes_from_file("bt.txt")
