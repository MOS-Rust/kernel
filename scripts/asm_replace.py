import os

def read_file(file):
    with open(file, 'r') as f:
        return f.read()
    
def write_file(file, content):
    with open(file, 'w') as f:
        f.write(content)
        
def walk_dir(dir, callback):
    for root, dirs, files in os.walk(dir):
        for file in files:
            callback(root, file)
        for dir in dirs:
            walk_dir(dir, callback)
    
def replace(root, file):
    if file.endswith('.asm'):
        asm_file = os.path.join(root, file)
        S_file = os.path.join(root.replace('asm', ''), file.replace('.asm', '.S'))
        source = read_file(asm_file)
        write_file(S_file, source)
        

if __name__ == '__main__':
    walk_dir('src', replace)