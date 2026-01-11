from pymongo import MongoClient
import sys

def clean_db():
    try:
        client = MongoClient("mongodb://localhost:27017")
        db = client["xhs_tools"]
        
        # Clean credentials collection
        credentials = db["credentials"]
        count = credentials.count_documents({})
        print(f"Found {count} credential records.")
        result = credentials.delete_many({})
        print(f"Deleted {result.deleted_count} from 'credentials'.")
        
        # Clean api_signatures collection
        signatures = db["api_signatures"]
        count = signatures.count_documents({})
        print(f"Found {count} signature records.")
        result = signatures.delete_many({})
        print(f"Deleted {result.deleted_count} from 'api_signatures'.")
        
        print("\n✅ Database cleaned successfully!")
        
    except Exception as e:
        print(f"Error cleaning database: {e}")
        sys.exit(1)

if __name__ == "__main__":
    clean_db()
