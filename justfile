default:
    @just --list

up: start-kind-cluster build-deps token-secret integration-tests keycloak ingress-controller wait-for-services ingress configure-keycloak bootstrap

fresh: stop up

stop:
    kind delete cluster --name kind

start-kind-cluster:
    kind create cluster --name kind --config=integration-tests/kind.yaml

build-deps:
    helm dependency build ./integration-tests/helm/setup

install-integration:
    helm upgrade --install --namespace default integration-tests integration-tests/helm/setup

key := `openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1`

integration-tests:
    helm upgrade --install --namespace default integration-tests integration-tests/helm/setup \
      --set-literal 'boxer-issuer.issuer.config.tokenSettings.key={{ key }}' \
      --set 'boxer-issuer.issuer.config.config.listenIp=0.0.0.0' \
      --set 'boxer-issuer.issuer.config.config.backend.kubernetes.resourceOwnerLabel=application/boxer-issuer' \
      --set 'boxer-issuer.issuer.replicas=1'

keycloak:
    helm upgrade --install keycloak oci://ghcr.io/codecentric/helm-charts/keycloakx \
      --set keycloak.username=admin \
      --set keycloak.password=admin \
      --values ./integration-tests/keycloak.yaml

ingress-controller:
    kubectl apply -f https://kind.sigs.k8s.io/examples/ingress/deploy-ingress-nginx.yaml

wait-for-services:
    kubectl rollout status deployment/ingress-nginx-controller --namespace ingress-nginx --timeout=180s
    kubectl rollout status statefulset/keycloak-keycloakx --timeout=180s
    kubectl rollout status deployment/boxer-issuer --timeout=180s

ingress:
    # Wait a bit for ingress controller to be ready to accept rules
    sleep 10
    # Create ingress rules for boxer-issuer and boxer-validator-nginx
    kubectl apply -f ./integration-tests/ingress.yaml

token-secret:
    kubectl create secret generic boxer-issuer-token-settings --from-literal=BOXER__TOKEN_SETTINGS__KEY='{{ key }}'

configure-keycloak:
    # Wait a bit for Keycloak to be ready to accept admin commands
    sleep 10

    # Create realm, client, and user for tests
    docker run --rm --network=host -v $(pwd)/integration-tests/terraform/keycloak:/tofu --workdir /tofu \
      ghcr.io/opentofu/opentofu:latest init
    docker run --rm --network=host -v $(pwd)/integration-tests/terraform/keycloak:/tofu --workdir /tofu \
      ghcr.io/opentofu/opentofu:latest plan
    docker run --rm --network=host -v $(pwd)/integration-tests/terraform/keycloak:/tofu --workdir /tofu \
      ghcr.io/opentofu/opentofu:latest apply -auto-approve

bootstrap:
    kubectl apply -f ./integration-tests/bootstrap/bootstrap.yaml
