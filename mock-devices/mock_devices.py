import pika
import json
import time
import random
from faker import Faker
from datetime import datetime

fake = Faker()
connection = pika.BlockingConnection(pika.ConnectionParameters('rabbitmq'))
channel = connection.channel()
channel.queue_declare(queue='machine_data')

def generate_data():
    return {
        "machine_id": f"MACH-{random.randint(1,50):03d}",
        "temperature": round(random.uniform(20, 120), 2),
        "vibration": round(random.uniform(0, 10), 2),
        "pressure": round(random.uniform(0, 15), 2),
        "status": random.choice(["RUNNING", "IDLE", "ERROR", "MAINTENANCE"]),
        "timestamp": datetime.utcnow().isoformat() + "Z"
    }

print("50 sanal makine başladı – veri gönderiyor...")
while True:
    data = generate_data()
    channel.basic_publish(exchange='', routing_key='machine_data', body=json.dumps(data))
    print(f"Sent: {data['machine_id']} – {data['status']} ({data['temperature']}°C)")
    time.sleep(0.5)