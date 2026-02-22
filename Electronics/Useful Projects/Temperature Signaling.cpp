#include <Arduino.h>
#include <DHT.h>

#define dhtPin 10
#define ledPin 17

DHT dht11(dhtPin, DHT11);

void setup() {
  
  Serial.begin(9600);
  pinMode(ledPin, OUTPUT);
  dht11.begin();
}

void loop() {
 
  delay(500);
  
  float t = dht11.readTemperature();
  float hum = dht11.readHumidity();

  if(isnan(t)) {

    Serial.println("Failed to Read Temp from DHT11");
    return;
  }

  float temp = dht11.computeHeatIndex(t, hum, false);

  if(temp > 20) {

    digitalWrite(ledPin, HIGH);
  } else { 

    digitalWrite(ledPin, LOW);
  }


  Serial.print(temp);
  Serial.print(" °C\n");
}

