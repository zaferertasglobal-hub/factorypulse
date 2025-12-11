FROM python:3.13-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY mock_devices.py .
CMD ["python", "mock_devices.py"]