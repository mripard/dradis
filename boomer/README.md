# Boomer

This application is to be used with Dradis to implement a CI system for HDMI. It runs on the DUT,
and will emit frames using the Linux Atomic KMS API.

Each of these frames will embed a QR Code, passing metadata for Dradis to check that the received
frames match the buffer initially sent.
