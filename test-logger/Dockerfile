FROM alpine:latest

# Copy the logger script
COPY logger.sh /logger.sh
RUN chmod +x /logger.sh

# Run the logger with unbuffered output
CMD ["sh", "-c", "exec /logger.sh"
