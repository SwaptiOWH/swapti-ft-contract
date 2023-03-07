### Swapti_FT

CONTRACT=dev-1678227171447-27568759288636
ACCOUNT=yairnava.testnet

Inicializar contrato

    near call $CONTRACT init_contract '{"owner_id": "'$CONTRACT'"}' --accountId $CONTRACT

Obtener propietario del contrato
    
    near view $CONTRACT get_owner_id

Cambiar propietario del contrato

    near call $CONTRACT set_owner_id '{"owner_id": "'$ACCOUNT'"}' --accountId $ACCOUNT

Obtener balance total de Swapti tokens
    
    near view $CONTRACT ft_total_supply

Obtener balance de una cuenta de Swapti tokens

    near view $CONTRACT ft_balance_of '{"account_id": "'$ACCOUNT'"}'

Agregar minero

    near call $ID add_minter '{"account_id": "'$ACCOUN'"}' --accountId $CONTRACT

Minar tokens (solo owner)

    near call $CONTRACT mint '{"account_id": "$ACCOUNT", "amount" : "1000000000000000000000000"}' --accountId $CONTRACT

Mostrar Swapti tokens en Wallet

    near call $CONTRACT ft_transfer '{"receiver_id": "'$ACCOUNT'", "amount":"0", "memo":""}' --accountId $ACCOUNT