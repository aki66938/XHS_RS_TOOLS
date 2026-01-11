from pymongo import MongoClient
c = MongoClient('mongodb://localhost:27017')
db = c['xhs_tools']
sigs = list(db['api_signatures'].find())
print('Signatures found:', len(sigs))
for s in sigs:
    print(f"\n=== {s.get('endpoint')} ===")
    print(f"  x_s: {s.get('x_s', '')[:50] if s.get('x_s') else 'EMPTY'}...")
    print(f"  x_t: {s.get('x_t', 'EMPTY')}")
    print(f"  x_s_common: {s.get('x_s_common', '')[:50] if s.get('x_s_common') else 'EMPTY'}...")
    print(f"  x_b3_traceid: {s.get('x_b3_traceid', 'EMPTY')}")
    print(f"  x_xray_traceid: {s.get('x_xray_traceid', 'EMPTY')}")
